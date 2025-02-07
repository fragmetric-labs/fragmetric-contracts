use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::modules::fund::FundAccount;
use crate::modules::reward::{RewardAccount, UserRewardAccount};
use crate::utils::{AccountLoaderExt, PDASeeds};

#[event_cpi]
#[derive(Accounts)]
pub struct AdminFundWrapAccountRewardAccountInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [FundAccount::WRAP_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub fund_wrap_account: SystemAccount<'info>,

    pub system_program: Program<'info, System>,

    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub receipt_token_program: Program<'info, Token2022>,

    #[account(
        associated_token::mint = receipt_token_mint,
        associated_token::token_program = receipt_token_program,
        associated_token::authority = fund_wrap_account,
    )]
    pub receipt_token_wrap_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = fund_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub fund_account: AccountLoader<'info, FundAccount>,

    #[account(
        mut,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = reward_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = reward_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub reward_account: AccountLoader<'info, RewardAccount>,

    #[account(
        init,
        payer = payer,
        seeds = [UserRewardAccount::SEED, receipt_token_mint.key().as_ref(), fund_wrap_account.key().as_ref()],
        bump,
        space = std::cmp::min(
            solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE,
            8 + std::mem::size_of::<UserRewardAccount>(),
        ),
    )]
    pub fund_wrap_account_reward_account: AccountLoader<'info, UserRewardAccount>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct AdminFundWrapAccountRewardAccountUpdateContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [FundAccount::WRAP_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub fund_wrap_account: SystemAccount<'info>,

    pub system_program: Program<'info, System>,

    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub receipt_token_program: Program<'info, Token2022>,

    #[account(
        associated_token::mint = receipt_token_mint,
        associated_token::token_program = receipt_token_program,
        associated_token::authority = fund_wrap_account,
    )]
    pub receipt_token_wrap_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = fund_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub fund_account: AccountLoader<'info, FundAccount>,

    #[account(
        mut,
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
        // DO NOT use has_one constraint, since reward_account is not safe yet
    )]
    pub fund_wrap_account_reward_account: AccountLoader<'info, UserRewardAccount>,
}
