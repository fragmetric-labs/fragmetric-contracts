use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions as instructions_sysvar;
use anchor_spl::token::Token;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::errors::ErrorCode;
use crate::modules::fund::{FundAccount, UserFundAccount};
use crate::modules::reward::{RewardAccount, UserRewardAccount};
use crate::utils::{AccountLoaderExt, PDASeeds};

#[event_cpi]
#[derive(Accounts)]
pub struct UserFundVaultReceiptTokenContext<'info> {
    pub user: Signer<'info>,

    pub receipt_token_program: Program<'info, Token2022>,

    pub vault_receipt_token_program: Interface<'info, TokenInterface>,

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = fund_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub fund_account: AccountLoader<'info, FundAccount>,

    #[account(
        seeds = [FundAccount::RESERVE_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub fund_reserve_account: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [UserFundAccount::SEED, receipt_token_mint.key().as_ref(), user.key().as_ref()],
        bump = user_fund_account.get_bump(),
        has_one = receipt_token_mint,
        has_one = user,
        constraint = user_fund_account.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub user_fund_account: Box<Account<'info, UserFundAccount>>,

    #[account(mut)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub vault_receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = vault_receipt_token_mint,
        associated_token::authority = user,
        associated_token::token_program = Token::id(),
    )]
    pub user_vault_receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = vault_receipt_token_mint,
        associated_token::authority = fund_reserve_account,
        associated_token::token_program = Token::id(),
    )]
    pub fund_vault_receipt_token_reserve_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
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

    #[account(
        mut,
        seeds = [UserRewardAccount::SEED, receipt_token_mint.key().as_ref(), user.key().as_ref()],
        bump = user_reward_account.get_bump()?,
        has_one = receipt_token_mint,
        has_one = user,
        constraint = user_reward_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub user_reward_account: AccountLoader<'info, UserRewardAccount>,

    /// CHECK: This is safe that checks it's ID
    #[account(address = instructions_sysvar::ID)]
    pub instructions_sysvar: UncheckedAccount<'info>,
}
