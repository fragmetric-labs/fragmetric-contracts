use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions as instructions_sysvar;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::modules::{fund::*, reward::*};
use crate::utils::{AccountLoaderExt, PDASeeds};

#[derive(Accounts)]
pub struct UserFundReceiptTokenAccountInitialContext<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub receipt_token_program: Program<'info, Token2022>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = user,
        associated_token::mint = receipt_token_mint,
        associated_token::token_program = receipt_token_program,
        associated_token::authority = user,
    )]
    pub user_receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
}

#[derive(Accounts)]
pub struct UserFundAccountInitialContext<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = user,
        seeds = [UserFundAccount::SEED, receipt_token_mint.key().as_ref(), user.key().as_ref()],
        bump,
        space = 8 + UserFundAccount::INIT_SPACE,
    )]
    pub user_fund_account: Box<Account<'info, UserFundAccount>>,
}

#[derive(Accounts)]
pub struct UserFundAccountUpdateContext<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [UserFundAccount::SEED, receipt_token_mint.key().as_ref(), user.key().as_ref()],
        bump = user_fund_account.bump(),
    )]
    pub user_fund_account: Box<Account<'info, UserFundAccount>>,
}

#[derive(Accounts)]
pub struct UserFundContext<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,

    // pub associated_token_program: Program<'info, AssociatedToken>,
    pub receipt_token_program: Program<'info, Token2022>,

    #[account(mut, address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [ReceiptTokenMintAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump = receipt_token_mint_authority.bump(),
    )]
    pub receipt_token_mint_authority: Account<'info, ReceiptTokenMintAuthority>,

    #[account(
        seeds = [ReceiptTokenLockAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump = receipt_token_lock_authority.bump(),
    )]
    pub receipt_token_lock_authority: Account<'info, ReceiptTokenLockAuthority>,

    #[account(
        mut,
        token::mint = receipt_token_mint,
        token::authority = receipt_token_lock_authority,
        token::token_program = receipt_token_program,
        seeds = [ReceiptTokenLockAuthority::TOKEN_ACCOUNT_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = receipt_token_mint,
        associated_token::token_program = receipt_token_program,
        associated_token::authority = user,
    )]
    pub user_receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

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
    pub instructions_sysvar: UncheckedAccount<'info>,
}

impl<'info> UserFundContext<'info> {
    pub fn check_user_sol_balance(&self, sol_amount: u64) -> Result<()> {
        require_gte!(self.user.lamports(), sol_amount);
        Ok(())
    }

    pub fn check_user_receipt_token_balance(&self, receipt_token_amount: u64) -> Result<()> {
        require_gte!(self.user_receipt_token_account.amount, receipt_token_amount);
        Ok(())
    }
}
