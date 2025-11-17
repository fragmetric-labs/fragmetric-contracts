use anchor_lang::prelude::*;
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::errors::ErrorCode;
use crate::modules::reward::*;
use crate::utils::{AccountLoaderExt, PDASeeds};

#[event_cpi]
#[derive(Accounts)]
pub struct UserRewardAccountInitOrUpdateContext<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub receipt_token_program: Program<'info, Token2022>,

    #[account(
        associated_token::mint = receipt_token_mint,
        associated_token::token_program = receipt_token_program,
        associated_token::authority = user,
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
}

#[event_cpi]
#[derive(Accounts)]
pub struct UserRewardContext<'info> {
    /// CHECK: This context does not require user's signature - it only updates user reward pools.
    pub user: UncheckedAccount<'info>,

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
        mut,
        seeds = [UserRewardAccount::SEED, receipt_token_mint.key().as_ref(), user.key().as_ref()],
        bump = user_reward_account.get_bump()?,
        has_one = receipt_token_mint,
        has_one = user,
        constraint = user_reward_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub user_reward_account: AccountLoader<'info, UserRewardAccount>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct UserRewardClaimContext<'info> {
    // Claim authority could be user or delegate
    pub claim_authority: Signer<'info>,

    /// CHECK: This context does not require user's signature, but claim authority's signature
    pub user: UncheckedAccount<'info>,

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
        mut,
        seeds = [UserRewardAccount::SEED, receipt_token_mint.key().as_ref(), user.key().as_ref()],
        bump = user_reward_account.get_bump()?,
        has_one = receipt_token_mint,
        has_one = user,
        constraint = user_reward_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub user_reward_account: AccountLoader<'info, UserRewardAccount>,

    #[account(
        seeds = [RewardAccount::RESERVE_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub reward_reserve_account: SystemAccount<'info>,

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
        token::mint = reward_token_mint,
        token::token_program = reward_token_program,
    )]
    pub destination_reward_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct UserRewardAccountDelegateContext<'info> {
    // Could be user or delegate
    pub delegate_authority: Signer<'info>,

    /// CHECK: This context does not require user's signature, but delegate authority's signature
    pub user: UncheckedAccount<'info>,

    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        associated_token::mint = receipt_token_mint,
        associated_token::authority = user,
        associated_token::token_program = Token2022::id(),
    )]
    pub user_receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = reward_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = reward_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub reward_account: AccountLoader<'info, RewardAccount>,

    #[account(
        mut,
        seeds = [UserRewardAccount::SEED, receipt_token_mint.key().as_ref(), user.key().as_ref()],
        bump = user_reward_account.get_bump()?,
        has_one = receipt_token_mint,
        has_one = user,
        constraint = user_reward_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub user_reward_account: AccountLoader<'info, UserRewardAccount>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct UserRewardAccountCloseContext<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: user might not have receipt_token_account...
    #[account(
        constraint = user_receipt_token_account.key()
            == get_associated_token_address_with_program_id(
                user.key,
                &receipt_token_mint.key(),
                &Token2022::id()
            ) @ error::ErrorCode::AccountNotAssociatedTokenAccount,
    )]
    pub user_receipt_token_account: UncheckedAccount<'info>,

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
        seeds = [UserRewardAccount::SEED, receipt_token_mint.key().as_ref(), user.key().as_ref()],
        bump = user_reward_account.get_bump()?,
        has_one = receipt_token_mint,
        has_one = user,
        constraint = user_reward_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub user_reward_account: AccountLoader<'info, UserRewardAccount>,
}
