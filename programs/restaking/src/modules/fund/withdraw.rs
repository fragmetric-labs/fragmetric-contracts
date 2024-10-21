use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, Token2022};
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::errors::ErrorCode;
use crate::events;
use crate::modules::reward::{self, RewardAccount, UserRewardAccount};
use crate::utils::PDASeeds;

use super::*;

pub fn request_withdrawal<'info>(
    user: &Signer<'info>,
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    receipt_token_mint_authority: &Account<'info, ReceiptTokenMintAuthority>,
    receipt_token_lock_account: &mut InterfaceAccount<'info, TokenAccount>,
    user_receipt_token_account: &mut InterfaceAccount<'info, TokenAccount>,
    fund_account: &mut FundAccount,
    reward_account: &mut RewardAccount,
    user_fund_account: &mut UserFundAccount,
    user_reward_account: &mut UserRewardAccount,
    user_reward_account_address: Pubkey,
    receipt_token_program: &Program<'info, Token2022>,
    receipt_token_amount: u64,
    current_slot: u64,
) -> Result<()> {
    let (batch_id, request_id) = user_fund_account
        .create_withdrawal_request(&mut fund_account.withdrawal_status, receipt_token_amount)?;

    lock_receipt_token(
        receipt_token_program,
        receipt_token_mint,
        receipt_token_mint_authority,
        receipt_token_lock_account,
        user,
        user_receipt_token_account,
        reward_account,
        user_fund_account,
        user_reward_account,
        user_reward_account_address,
        receipt_token_amount,
        current_slot,
    )?;

    emit!(events::UserRequestedWithdrawalFromFund {
        user: user.key(),
        user_receipt_token_account: user_receipt_token_account.key(),
        user_fund_account: user_fund_account.clone(),
        batch_id,
        request_id,
        receipt_token_mint: receipt_token_mint.key(),
        requested_receipt_token_amount: receipt_token_amount,
    });

    Ok(())
}

pub fn cancel_withdrawal_request<'info>(
    user: &Signer<'info>,
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    receipt_token_mint_authority: &Account<'info, ReceiptTokenMintAuthority>,
    receipt_token_lock_account: &mut InterfaceAccount<'info, TokenAccount>,
    receipt_token_lock_authority: &Account<'info, ReceiptTokenLockAuthority>,
    user_receipt_token_account: &mut InterfaceAccount<'info, TokenAccount>,
    fund_account: &mut FundAccount,
    reward_account: &mut RewardAccount,
    user_fund_account: &mut UserFundAccount,
    user_reward_account: &mut UserRewardAccount,
    user_reward_account_address: Pubkey,
    receipt_token_program: &Program<'info, Token2022>,
    request_id: u64,
    current_slot: u64,
) -> Result<()> {
    let request = user_fund_account
        .cancel_withdrawal_request(&mut fund_account.withdrawal_status, request_id)?;

    unlock_receipt_token(
        receipt_token_program,
        receipt_token_mint,
        receipt_token_mint_authority,
        receipt_token_lock_account,
        receipt_token_lock_authority,
        user_receipt_token_account,
        reward_account,
        user_fund_account,
        user_reward_account,
        user_reward_account_address,
        request.receipt_token_amount,
        current_slot,
    )?;

    emit!(events::UserCanceledWithdrawalRequestFromFund {
        user: user.key(),
        user_receipt_token_account: user_receipt_token_account.key(),
        user_fund_account: user_fund_account.clone(),
        request_id,
        receipt_token_mint: receipt_token_mint.key(),
        requested_receipt_token_amount: request.receipt_token_amount,
    });

    Ok(())
}

pub fn withdraw(
    user: &Signer,
    receipt_token_mint: &Mint,
    fund_account: &mut Account<FundAccount>,
    user_fund_account: &mut UserFundAccount,
    request_id: u64,
) -> Result<()> {
    let request = user_fund_account
        .pop_completed_withdrawal_request(&mut fund_account.withdrawal_status, request_id)?;

    let (sol_withdraw_amount, sol_fee_amount) =
        transfer_sol_from_fund_to_user(user, fund_account, request.receipt_token_amount)?;

    emit!(events::UserWithdrewSOLFromFund {
        receipt_token_mint: fund_account.receipt_token_mint,
        fund_account: FundAccountInfo::new(
            fund_account.as_ref(),
            fund_account.receipt_token_sol_value_per_token(
                receipt_token_mint.decimals,
                receipt_token_mint.supply,
            )?,
            receipt_token_mint.supply
        ),
        request_id,
        user_fund_account: user_fund_account.clone(),
        user: user.key(),
        burnt_receipt_token_amount: request.receipt_token_amount,
        withdrawn_sol_amount: sol_withdraw_amount,
        deducted_sol_fee_amount: sol_fee_amount,
    });

    Ok(())
}

fn lock_receipt_token<'info>(
    receipt_token_program: &Program<'info, Token2022>,
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    receipt_token_mint_authority: &Account<'info, ReceiptTokenMintAuthority>,
    receipt_token_lock_account: &mut InterfaceAccount<'info, TokenAccount>,
    user: &Signer<'info>,
    user_receipt_token_account: &mut InterfaceAccount<'info, TokenAccount>,
    reward_account: &mut RewardAccount,
    user_fund_account: &mut UserFundAccount,
    user_reward_account: &mut UserRewardAccount,
    user_reward_account_address: Pubkey,
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
            &[receipt_token_mint_authority.signer_seeds().as_ref()],
        ),
        receipt_token_amount,
    )
    .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))?;

    receipt_token_mint.reload()?;
    receipt_token_lock_account.reload()?;
    user_receipt_token_account.reload()?;
    user_fund_account.set_receipt_token_amount(user_receipt_token_account.amount);

    reward::update_reward_pools_token_allocation(
        reward_account,
        Some(user_reward_account),
        None,
        vec![user_reward_account_address],
        receipt_token_mint.key(),
        receipt_token_amount,
        None,
        current_slot,
    )
}

fn unlock_receipt_token<'info>(
    receipt_token_program: &Program<'info, Token2022>,
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    receipt_token_mint_authority: &Account<'info, ReceiptTokenMintAuthority>,
    receipt_token_lock_account: &mut InterfaceAccount<'info, TokenAccount>,
    receipt_token_lock_authority: &Account<'info, ReceiptTokenLockAuthority>,
    user_receipt_token_account: &mut InterfaceAccount<'info, TokenAccount>,
    reward_account: &mut RewardAccount,
    user_fund_account: &mut UserFundAccount,
    user_reward_account: &mut UserRewardAccount,
    user_reward_account_address: Pubkey,
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
            &[receipt_token_lock_authority.signer_seeds().as_ref()],
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
            &[receipt_token_mint_authority.signer_seeds().as_ref()],
        ),
        receipt_token_amount,
    )
    .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))?;

    receipt_token_mint.reload()?;
    receipt_token_lock_account.reload()?;
    user_receipt_token_account.reload()?;
    user_fund_account.set_receipt_token_amount(user_receipt_token_account.amount);

    reward::update_reward_pools_token_allocation(
        reward_account,
        None,
        Some(user_reward_account),
        vec![user_reward_account_address],
        receipt_token_mint.key(),
        receipt_token_amount,
        None,
        current_slot,
    )
}

/// Returns (sol_transferred_amount, sol_fee_amount)
fn transfer_sol_from_fund_to_user(
    user: &Signer,
    fund_account: &mut Account<FundAccount>,
    receipt_token_withdraw_amount: u64,
) -> Result<(u64, u64)> {
    let sol_amount = fund_account
        .withdrawal_status
        .reserved_fund
        .calculate_sol_amount_for_receipt_token_amount(receipt_token_withdraw_amount)?;
    let sol_fee_amount = fund_account
        .withdrawal_status
        .calculate_sol_withdrawal_fee(sol_amount)?;
    let sol_transferred_amount = sol_amount
        .checked_sub(sol_fee_amount)
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

    fund_account.withdrawal_status.withdraw(
        sol_amount,
        sol_fee_amount,
        receipt_token_withdraw_amount,
    )?;
    fund_account.sub_lamports(sol_transferred_amount)?;
    user.add_lamports(sol_transferred_amount)?;

    Ok((sol_transferred_amount, sol_fee_amount))
}
