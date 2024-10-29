use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, Token2022};
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::errors::ErrorCode;
use crate::events;
use crate::modules::fund::*;
use crate::modules::reward::{self, RewardAccount, UserRewardAccount};
use crate::utils::PDASeeds;

pub fn process_request_withdrawal<'info>(
    user: &Signer<'info>,
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    receipt_token_mint_authority: &Account<'info, ReceiptTokenMintAuthority>,
    receipt_token_lock_account: &mut InterfaceAccount<'info, TokenAccount>,
    user_receipt_token_account: &mut InterfaceAccount<'info, TokenAccount>,
    fund_account: &mut Account<FundAccount>,
    reward_account: &mut AccountLoader<RewardAccount>,
    user_fund_account: &mut Account<UserFundAccount>,
    user_reward_account: &mut AccountLoader<UserRewardAccount>,
    receipt_token_program: &Program<'info, Token2022>,
    receipt_token_amount: u64,
    current_slot: u64,
    current_timestamp: i64,
) -> Result<()> {
    require_gte!(user_receipt_token_account.amount, receipt_token_amount);

    fund_account.withdrawal.assert_withdrawal_enabled()?;

    let (batch_id, request_id) = user_fund_account.create_withdrawal_request(
        &mut fund_account.withdrawal,
        receipt_token_amount,
        current_timestamp,
    )?;

    lock_receipt_token(
        user,
        receipt_token_mint,
        receipt_token_mint_authority,
        receipt_token_lock_account,
        user_receipt_token_account,
        reward_account,
        user_fund_account,
        user_reward_account,
        receipt_token_program,
        receipt_token_amount,
        current_slot,
    )?;

    emit!(events::UserRequestedWithdrawalFromFund {
        user: user.key(),
        user_receipt_token_account: user_receipt_token_account.key(),
        user_fund_account: Clone::clone(user_fund_account),
        batch_id,
        request_id,
        receipt_token_mint: receipt_token_mint.key(),
        requested_receipt_token_amount: receipt_token_amount,
    });

    Ok(())
}

pub fn process_cancel_withdrawal_request<'info>(
    user: &Signer<'info>,
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    receipt_token_mint_authority: &Account<'info, ReceiptTokenMintAuthority>,
    receipt_token_lock_account: &mut InterfaceAccount<'info, TokenAccount>,
    receipt_token_lock_authority: &Account<'info, ReceiptTokenLockAuthority>,
    user_receipt_token_account: &mut InterfaceAccount<'info, TokenAccount>,
    fund_account: &mut Account<FundAccount>,
    reward_account: &mut AccountLoader<RewardAccount>,
    user_fund_account: &mut Account<UserFundAccount>,
    user_reward_account: &mut AccountLoader<UserRewardAccount>,
    receipt_token_program: &Program<'info, Token2022>,
    request_id: u64,
    current_slot: u64,
) -> Result<()> {
    let receipt_token_amount =
        user_fund_account.cancel_withdrawal_request(&mut fund_account.withdrawal, request_id)?;

    unlock_receipt_token(
        receipt_token_mint,
        receipt_token_mint_authority,
        receipt_token_lock_account,
        receipt_token_lock_authority,
        user_receipt_token_account,
        reward_account,
        user_fund_account,
        user_reward_account,
        receipt_token_program,
        receipt_token_amount,
        current_slot,
    )?;

    emit!(events::UserCanceledWithdrawalRequestFromFund {
        user: user.key(),
        user_receipt_token_account: user_receipt_token_account.key(),
        user_fund_account: Clone::clone(user_fund_account),
        request_id,
        receipt_token_mint: receipt_token_mint.key(),
        requested_receipt_token_amount: receipt_token_amount,
    });

    Ok(())
}

pub fn process_withdraw(
    user: &Signer,
    receipt_token_mint: &InterfaceAccount<Mint>,
    fund_account: &mut Account<FundAccount>,
    user_fund_account: &mut Account<UserFundAccount>,
    request_id: u64,
) -> Result<()> {
    let (sol_withdraw_amount, sol_fee_amount, receipt_token_withdraw_amount) =
        user_fund_account.claim_withdrawal_request(&mut fund_account.withdrawal, request_id)?;

    fund_account.sub_lamports(sol_withdraw_amount)?;
    user.add_lamports(sol_withdraw_amount)?;

    emit!(events::UserWithdrewSOLFromFund {
        receipt_token_mint: fund_account.receipt_token_mint,
        fund_account: FundAccountInfo::from(
            fund_account,
            receipt_token_mint,
        ),
        request_id,
        user_fund_account: Clone::clone(user_fund_account),
        user: user.key(),
        burnt_receipt_token_amount: receipt_token_withdraw_amount,
        withdrawn_sol_amount: sol_withdraw_amount,
        deducted_sol_fee_amount: sol_fee_amount,
    });

    Ok(())
}

fn lock_receipt_token<'info>(
    user: &Signer<'info>,
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    receipt_token_mint_authority: &Account<'info, ReceiptTokenMintAuthority>,
    receipt_token_lock_account: &mut InterfaceAccount<'info, TokenAccount>,
    user_receipt_token_account: &mut InterfaceAccount<'info, TokenAccount>,
    reward_account: &mut AccountLoader<RewardAccount>,
    user_fund_account: &mut Account<UserFundAccount>,
    user_reward_account: &mut AccountLoader<UserRewardAccount>,
    receipt_token_program: &Program<'info, Token2022>,
    receipt_token_amount: u64,
    current_slot: u64,
) -> Result<()> {
    token_2022::burn(
        CpiContext::new(
            receipt_token_program.to_account_info(),
            token_2022::Burn {
                mint: receipt_token_mint.to_account_info(),
                from: user_receipt_token_account.to_account_info(),
                authority: user.to_account_info(),
            },
        ),
        receipt_token_amount,
    )
    .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))?;

    token_2022::mint_to(
        CpiContext::new_with_signer(
            receipt_token_program.to_account_info(),
            token_2022::MintTo {
                mint: receipt_token_mint.to_account_info(),
                to: receipt_token_lock_account.to_account_info(),
                authority: receipt_token_mint_authority.to_account_info(),
            },
            &[receipt_token_mint_authority.get_signer_seeds().as_ref()],
        ),
        receipt_token_amount,
    )
    .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))?;

    receipt_token_mint.reload()?;
    receipt_token_lock_account.reload()?;
    user_receipt_token_account.reload()?;
    user_fund_account.receipt_token_amount = user_receipt_token_account.amount;

    reward::update_reward_pools_token_allocation(
        &mut *reward_account.load_mut()?,
        Some(&mut *user_reward_account.load_mut()?),
        None,
        vec![user_reward_account.key()],
        receipt_token_mint.key(),
        receipt_token_amount,
        None,
        current_slot,
    )
}

fn unlock_receipt_token<'info>(
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    receipt_token_mint_authority: &Account<'info, ReceiptTokenMintAuthority>,
    receipt_token_lock_account: &mut InterfaceAccount<'info, TokenAccount>,
    receipt_token_lock_authority: &Account<'info, ReceiptTokenLockAuthority>,
    user_receipt_token_account: &mut InterfaceAccount<'info, TokenAccount>,
    reward_account: &mut AccountLoader<RewardAccount>,
    user_fund_account: &mut Account<UserFundAccount>,
    user_reward_account: &mut AccountLoader<UserRewardAccount>,
    receipt_token_program: &Program<'info, Token2022>,
    receipt_token_amount: u64,
    current_slot: u64,
) -> Result<()> {
    token_2022::burn(
        CpiContext::new_with_signer(
            receipt_token_program.to_account_info(),
            token_2022::Burn {
                mint: receipt_token_mint.to_account_info(),
                from: receipt_token_lock_account.to_account_info(),
                authority: receipt_token_lock_authority.to_account_info(),
            },
            &[receipt_token_lock_authority.get_signer_seeds().as_ref()],
        ),
        receipt_token_amount,
    )
    .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))?;

    token_2022::mint_to(
        CpiContext::new_with_signer(
            receipt_token_program.to_account_info(),
            token_2022::MintTo {
                mint: receipt_token_mint.to_account_info(),
                to: user_receipt_token_account.to_account_info(),
                authority: receipt_token_mint_authority.to_account_info(),
            },
            &[receipt_token_mint_authority.get_signer_seeds().as_ref()],
        ),
        receipt_token_amount,
    )
    .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))?;

    receipt_token_mint.reload()?;
    receipt_token_lock_account.reload()?;
    user_receipt_token_account.reload()?;
    user_fund_account.receipt_token_amount = user_receipt_token_account.amount;

    reward::update_reward_pools_token_allocation(
        &mut *reward_account.load_mut()?,
        None,
        Some(&mut *user_reward_account.load_mut()?),
        vec![user_reward_account.key()],
        receipt_token_mint.key(),
        receipt_token_amount,
        None,
        current_slot,
    )
}
