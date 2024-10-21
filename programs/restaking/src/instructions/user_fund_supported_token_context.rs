use anchor_lang::{prelude::*, solana_program::sysvar::instructions as instructions_sysvar};
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::modules::{fund::*, reward::*};
use crate::utils::{AccountLoaderExt, PDASeeds};

#[derive(Accounts)]
pub struct UserFundSupportedTokenContext<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub receipt_token_program: Program<'info, Token2022>,

    pub supported_token_program: Interface<'info, TokenInterface>,

    #[account(mut, address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [ReceiptTokenMintAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump = receipt_token_mint_authority.bump(),
    )]
    pub receipt_token_mint_authority: Box<Account<'info, ReceiptTokenMintAuthority>>,

    #[account(
        mut,
        associated_token::mint = receipt_token_mint,
        associated_token::token_program = receipt_token_program,
        associated_token::authority = user,
    )]
    pub user_receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub supported_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [SupportedTokenAuthority::SEED, receipt_token_mint.key().as_ref(), supported_token_mint.key().as_ref()],
        bump = supported_token_authority.bump(),
    )]
    pub supported_token_authority: Box<Account<'info, SupportedTokenAuthority>>,

    #[account(
        mut,
        token::mint = supported_token_mint,
        token::token_program = supported_token_program,
        token::authority = supported_token_authority,
        seeds = [SupportedTokenAuthority::TOKEN_ACCOUNT_SEED, receipt_token_mint.key().as_ref(), supported_token_mint.key().as_ref()],
        bump,
    )]
    pub supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = supported_token_mint,
        token::token_program = supported_token_program,
        token::authority = user.key(),
    )]
    pub user_supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.bump(),
        constraint = fund_account.is_latest_version() @ ErrorCode::InvalidDataVersionError,
    )]
    pub fund_account: Box<Account<'info, FundAccount>>,

    #[account(
        mut,
        seeds = [UserFundAccount::SEED, receipt_token_mint.key().as_ref(), user.key().as_ref()],
        bump = user_fund_account.bump(),
    )]
    pub user_fund_account: Box<Account<'info, UserFundAccount>>,

    #[account(
        mut,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = reward_account.bump()?,
        constraint = reward_account.load()?.is_latest_version() @ ErrorCode::InvalidDataVersionError,
    )]
    pub reward_account: AccountLoader<'info, RewardAccount>,

    #[account(
        mut,
        seeds = [UserRewardAccount::SEED, receipt_token_mint.key().as_ref(), user.key().as_ref()],
        bump = user_reward_account.bump()?,
        constraint = user_reward_account.load()?.is_latest_version() @ ErrorCode::InvalidDataVersionError,
    )]
    pub user_reward_account: AccountLoader<'info, UserRewardAccount>,

    /// CHECK: This is safe that checks it's ID
    #[account(address = instructions_sysvar::ID)]
    pub instruction_sysvar: UncheckedAccount<'info>,
}

impl<'info> UserFundSupportedTokenContext<'info> {
    pub fn check_user_supported_token_balance(&self, amount: u64) -> Result<()> {
        require_gte!(self.user_supported_token_account.amount, amount);
        Ok(())
    }
}
