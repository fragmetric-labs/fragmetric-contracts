use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::modules::fund::*;
use crate::modules::normalization::NormalizedTokenPoolAccount;
use crate::utils::{AccountLoaderExt, PDASeeds};

#[event_cpi]
#[derive(Accounts)]
pub struct FundManagerFundSupportedTokenContext<'info> {
    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

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
        seeds = [FundAccount::TREASURY_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub fund_treasury_account: SystemAccount<'info>,

    pub supported_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub supported_token_program: Interface<'info, TokenInterface>,

    #[account(
        associated_token::mint = supported_token_mint,
        associated_token::authority = fund_reserve_account,
        associated_token::token_program = supported_token_program,
    )]
    pub supported_token_reserve_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        associated_token::mint = supported_token_mint,
        associated_token::authority = fund_treasury_account,
        associated_token::token_program = supported_token_program,
    )]
    pub supported_token_treasury_account: Box<InterfaceAccount<'info, TokenAccount>>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct FundManagerFundSupportedTokenRemoveContext<'info> {
    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = fund_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub fund_account: AccountLoader<'info, FundAccount>,

    pub normalized_token_mint: Option<Box<InterfaceAccount<'info, Mint>>>,

    // #[account(
    //     mut,
    //     seeds = [NormalizedTokenPoolAccount::SEED, normalized_token_mint.key().as_ref()],
    //     bump = normalized_token_pool_account.get_bump(),
    //     has_one = normalized_token_mint,
    //     constraint = normalized_token_pool_account.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    // )]
    #[account(mut)]
    pub normalized_token_pool_account: Option<Box<Account<'info, NormalizedTokenPoolAccount>>>,

    pub supported_token_mint: Box<InterfaceAccount<'info, Mint>>,
}
