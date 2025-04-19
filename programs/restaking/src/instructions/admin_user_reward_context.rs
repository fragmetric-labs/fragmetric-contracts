use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::modules::reward::{RewardAccount, UserRewardAccount};
use crate::utils::{AccountLoaderExt, PDASeeds};

#[event_cpi]
#[derive(Accounts)]
pub struct AdminUserRewardAccountInitOrUpdateContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    /// CHECK: Third party account or someone else which could be pda or wallet or token account, etc.
    pub user: UncheckedAccount<'info>,

    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        associated_token::mint = receipt_token_mint,
        associated_token::authority = user,
        associated_token::token_program = Token2022::id(),
    )]
    pub user_receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = reward_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = reward_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub reward_account: AccountLoader<'info, RewardAccount>,

    /// CHECK: This account is treated as UncheckedAccount to determine whether to init or update.
    #[account(
        mut,
        seeds = [UserRewardAccount::SEED, receipt_token_mint.key().as_ref(), user.key().as_ref()],
        bump,
    )]
    pub user_reward_account: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

/// TODO remove this ix after migration (v0.6.0)
#[derive(Accounts)]
pub struct AdminDelegateFundWrapAccountRewardAccount<'info> {
    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [crate::modules::fund::FundAccount::WRAP_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub fund_wrap_account: SystemAccount<'info>,

    #[account(
        mut,
        associated_token::mint = receipt_token_mint,
        associated_token::authority = fund_wrap_account,
        associated_token::token_program = Token2022::id(),
    )]
    pub receipt_token_wrap_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = reward_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = reward_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub reward_account: AccountLoader<'info, RewardAccount>,

    #[account(
        mut,
        seeds = [UserRewardAccount::SEED, receipt_token_mint.key().as_ref(), fund_wrap_account.key().as_ref()],
        bump = fund_wrap_account_reward_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = fund_wrap_account_reward_account.load()?.user == fund_wrap_account.key() @ error::ErrorCode::ConstraintHasOne,
        constraint = fund_wrap_account_reward_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub fund_wrap_account_reward_account: AccountLoader<'info, UserRewardAccount>,
}
