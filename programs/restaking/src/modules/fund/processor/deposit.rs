use anchor_lang::{prelude::*, system_program};
use anchor_spl::token_2022::{self, Token2022};
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface};

use crate::errors::ErrorCode;
use crate::events;
use crate::modules::ed25519;
use crate::modules::fund::*;
use crate::modules::reward::{self, RewardAccount, UserRewardAccount};
use crate::utils::PDASeeds;

pub fn process_deposit_sol<'info>(
    user: &Signer<'info>,
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    receipt_token_mint_authority: &Account<'info, ReceiptTokenMintAuthority>,
    user_receipt_token_account: &mut InterfaceAccount<'info, TokenAccount>,
    fund_account: &mut Account<'info, FundAccount>,
    reward_account: &mut AccountLoader<RewardAccount>,
    user_fund_account: &mut Account<UserFundAccount>,
    user_reward_account: &mut AccountLoader<UserRewardAccount>,
    system_program: &Program<'info, System>,
    receipt_token_program: &Program<'info, Token2022>,
    instructions_sysvar: &AccountInfo,
    pricing_sources: &'info [AccountInfo<'info>],
    sol_amount: u64,
    metadata: Option<DepositMetadata>,
    current_slot: u64,
    current_timestamp: i64,
) -> Result<()> {
    require_gte!(user.lamports(), sol_amount);

    let (wallet_provider, contribution_accrual_rate) =
        verify_deposit_metadata(metadata, instructions_sysvar, current_timestamp)?;

    update_asset_prices(fund_account, pricing_sources)?;
    let receipt_token_mint_amount =
        receipt_token_mint_amount_for(receipt_token_mint, fund_account, sol_amount)?;

    mint_receipt_token_to_user(
        receipt_token_mint,
        receipt_token_mint_authority,
        user_receipt_token_account,
        reward_account,
        user_fund_account,
        user_reward_account,
        receipt_token_program,
        receipt_token_mint_amount,
        contribution_accrual_rate,
        current_slot,
    )?;

    transfer_sol_from_user_to_fund(user, fund_account, system_program, sol_amount)?;

    emit!(events::UserDepositedSOLToFund {
        user: user.key(),
        user_receipt_token_account: user_receipt_token_account.key(),
        user_fund_account: Clone::clone(user_fund_account),
        deposited_sol_amount: sol_amount,
        receipt_token_mint: receipt_token_mint.key(),
        minted_receipt_token_amount: receipt_token_mint_amount,
        wallet_provider,
        contribution_accrual_rate,
        fund_account: FundAccountInfo::from(
            fund_account,
            receipt_token_mint,
        ),
    });

    Ok(())
}

pub fn process_deposit_supported_token<'info>(
    user: &Signer<'info>,
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    receipt_token_mint_authority: &Account<'info, ReceiptTokenMintAuthority>,
    user_receipt_token_account: &mut InterfaceAccount<'info, TokenAccount>,
    supported_token_mint: &InterfaceAccount<'info, Mint>,
    supported_token_account: &InterfaceAccount<'info, TokenAccount>,
    user_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
    fund_account: &mut Account<FundAccount>,
    reward_account: &mut AccountLoader<RewardAccount>,
    user_fund_account: &mut Account<UserFundAccount>,
    user_reward_account: &mut AccountLoader<UserRewardAccount>,
    receipt_token_program: &Program<'info, Token2022>,
    supported_token_program: &Interface<'info, TokenInterface>,
    instructions_sysvar: &AccountInfo,
    pricing_sources: &'info [AccountInfo<'info>],
    supported_token_amount: u64,
    metadata: Option<DepositMetadata>,
    current_slot: u64,
    current_timestamp: i64,
) -> Result<()> {
    require_gte!(user_supported_token_account.amount, supported_token_amount);

    let (wallet_provider, contribution_accrual_rate) =
        verify_deposit_metadata(metadata, instructions_sysvar, current_timestamp)?;

    update_asset_prices(fund_account, pricing_sources)?;
    let receipt_token_mint_amount = receipt_token_mint_amount_for(
        receipt_token_mint,
        fund_account,
        fund_account
            .get_supported_token(supported_token_mint.key())?
            .get_token_amount_as_sol(supported_token_amount)?,
    )?;

    mint_receipt_token_to_user(
        receipt_token_mint,
        receipt_token_mint_authority,
        user_receipt_token_account,
        reward_account,
        user_fund_account,
        user_reward_account,
        receipt_token_program,
        receipt_token_mint_amount,
        contribution_accrual_rate,
        current_slot,
    )?;

    transfer_supported_token_from_user_to_fund(
        user,
        supported_token_mint,
        supported_token_account,
        user_supported_token_account,
        fund_account,
        supported_token_program,
        supported_token_amount,
    )?;

    emit!(events::UserDepositedSupportedTokenToFund {
        user: user.key(),
        user_receipt_token_account: user_receipt_token_account.key(),
        user_fund_account: Clone::clone(user_fund_account),
        supported_token_mint: supported_token_mint.key(),
        supported_token_user_account: user_supported_token_account.key(),
        deposited_supported_token_amount: supported_token_amount,
        receipt_token_mint: receipt_token_mint.key(),
        minted_receipt_token_amount: receipt_token_mint_amount,
        wallet_provider,
        contribution_accrual_rate,
        fund_account: FundAccountInfo::from(
            fund_account,
            receipt_token_mint,
        ),
    });

    Ok(())
}

/// Returns (wallet_provider, contribution_accrual_rate)
fn verify_deposit_metadata(
    metadata: Option<DepositMetadata>,
    instructions_sysvar: &AccountInfo,
    current_timestamp: i64,
) -> Result<(Option<String>, Option<u8>)> {
    if let Some(metadata) = &metadata {
        ed25519::verify_preceding_ed25519_instruction(
            instructions_sysvar,
            metadata.try_to_vec()?.as_slice(),
        )?;
        metadata.assert_metadata_not_exipred(current_timestamp)?;
    }
    Ok(metadata.map(DepositMetadata::split).unzip())
}

fn transfer_sol_from_user_to_fund<'info>(
    user: &Signer<'info>,
    fund_account: &mut Account<'info, FundAccount>,
    system_program: &Program<'info, System>,
    sol_amount: u64,
) -> Result<()> {
    fund_account.deposit_sol(sol_amount)?;
    system_program::transfer(
        CpiContext::new(
            system_program.to_account_info(),
            system_program::Transfer {
                from: user.to_account_info(),
                to: fund_account.to_account_info(),
            },
        ),
        sol_amount,
    )
}

fn transfer_supported_token_from_user_to_fund<'info>(
    user: &Signer<'info>,
    supported_token_mint: &InterfaceAccount<'info, Mint>,
    supported_token_account: &InterfaceAccount<'info, TokenAccount>,
    user_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
    fund_account: &mut Account<FundAccount>,
    supported_token_program: &Interface<'info, TokenInterface>,
    supported_token_amount: u64,
) -> Result<()> {
    fund_account
        .get_supported_token_mut(supported_token_mint.key())?
        .deposit_token(supported_token_amount)?;
    token_interface::transfer_checked(
        CpiContext::new(
            supported_token_program.to_account_info(),
            token_interface::TransferChecked {
                from: user_supported_token_account.to_account_info(),
                to: supported_token_account.to_account_info(),
                mint: supported_token_mint.to_account_info(),
                authority: user.to_account_info(),
            },
        ),
        supported_token_amount,
        supported_token_mint.decimals,
    )
        .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))
}

fn receipt_token_mint_amount_for(
    receipt_token_mint: &InterfaceAccount<Mint>,
    fund_account: &Account<FundAccount>,
    sol_amount: u64,
) -> Result<u64> {
    crate::utils::get_proportional_amount(
        sol_amount,
        receipt_token_mint.supply,
        fund_account.get_assets_total_amount_as_sol()?,
    )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
}

fn mint_receipt_token_to_user<'info>(
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    receipt_token_mint_authority: &Account<'info, ReceiptTokenMintAuthority>,
    user_receipt_token_account: &mut InterfaceAccount<'info, TokenAccount>,
    reward_account: &mut AccountLoader<RewardAccount>,
    user_fund_account: &mut Account<UserFundAccount>,
    user_reward_account: &mut AccountLoader<UserRewardAccount>,
    receipt_token_program: &Program<'info, Token2022>,
    receipt_token_mint_amount: u64,
    contribution_accrual_rate: Option<u8>,
    current_slot: u64,
) -> Result<()> {
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
        receipt_token_mint_amount,
    )
        .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))?;
    receipt_token_mint.reload()?;
    user_fund_account.sync_receipt_token_amount(user_receipt_token_account)?;

    reward::update_reward_pools_token_allocation(
        &mut *reward_account.load_mut()?,
        None,
        Some(&mut *user_reward_account.load_mut()?),
        vec![user_reward_account.key()],
        receipt_token_mint.key(),
        receipt_token_mint_amount,
        contribution_accrual_rate,
        current_slot,
    )
}
