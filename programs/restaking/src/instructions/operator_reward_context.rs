use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::PROGRAM_REVENUE_ADDRESS;
use crate::errors::ErrorCode;
use crate::modules::reward::*;
use crate::utils::{AccountLoaderExt, PDASeeds};

#[event_cpi]
#[derive(Accounts)]
pub struct OperatorRewardContext<'info> {
    pub operator: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = reward_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = reward_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub reward_account: AccountLoader<'info, RewardAccount>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct OperatorRewardClaimContext<'info> {
    pub operator: Signer<'info>,

    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = reward_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = reward_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub reward_account: AccountLoader<'info, RewardAccount>,

    #[account(
        seeds = [RewardAccount::RESERVE_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub reward_reserve_account: SystemAccount<'info>,

    /// CHECK: program revenue wallet
    #[account(address = PROGRAM_REVENUE_ADDRESS)]
    pub program_revenue_account: UncheckedAccount<'info>,

    pub reward_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub reward_token_program: Interface<'info, TokenInterface>,

    #[account(
        mut,
        associated_token::mint = reward_token_mint,
        associated_token::authority = reward_reserve_account,
        associated_token::token_program = reward_token_program,
    )]
    pub reward_token_reserve_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = reward_token_mint,
        associated_token::authority = program_revenue_account,
        associated_token::token_program = reward_token_program,
    )]
    pub program_reward_token_revenue_account: Box<InterfaceAccount<'info, TokenAccount>>,
}
