use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::modules::fund::FundAccount;
use crate::modules::reward::*;
use crate::utils::{AccountLoaderExt, PDASeeds};

#[event_cpi]
#[derive(Accounts)]
pub struct FundManagerFundWrappedTokenInitialContext<'info> {
    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

    #[account(
        seeds = [FundAccount::WRAP_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub fund_wrap_account: SystemAccount<'info>,

    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub receipt_token_program: Program<'info, Token2022>,

    #[account(
        associated_token::mint = receipt_token_mint,
        associated_token::token_program = receipt_token_program,
        associated_token::authority = fund_wrap_account,
    )]
    pub receipt_token_wrap_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        mint::token_program = wrapped_token_program,
        constraint = receipt_token_wrap_account.amount == wrapped_token_mint.supply,
    )]
    pub wrapped_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub wrapped_token_program: Program<'info, Token>,

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = fund_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub fund_account: AccountLoader<'info, FundAccount>,

    #[account(
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = reward_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = reward_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub reward_account: AccountLoader<'info, RewardAccount>,

    #[account(
        seeds = [UserRewardAccount::SEED, receipt_token_mint.key().as_ref(), fund_wrap_account.key().as_ref()],
        bump = fund_wrap_account_reward_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = fund_wrap_account_reward_account.load()?.user == fund_wrap_account.key() @ error::ErrorCode::ConstraintHasOne,
        constraint = fund_wrap_account_reward_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub fund_wrap_account_reward_account: AccountLoader<'info, UserRewardAccount>,
}
