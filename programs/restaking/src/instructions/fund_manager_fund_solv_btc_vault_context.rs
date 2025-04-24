use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::modules::fund::FundAccount;
use crate::utils::{AccountLoaderExt, PDASeeds};

/// TODO/v0.7.0 merge with jito restaking vault context
#[event_cpi]
#[derive(Accounts)]
pub struct FundManagerFundSolvBTCVaultInitialContext<'info> {
    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

    pub system_program: Program<'info, System>,

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

    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: TODO
    pub vault_program: UncheckedAccount<'info>,

    /// CHECK: TODO
    pub vault_account: UncheckedAccount<'info>,

    pub vault_receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub vault_supported_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        associated_token::mint = vault_receipt_token_mint,
        associated_token::authority = fund_reserve_account,
        associated_token::token_program = Token::id(),
    )]
    pub fund_vault_receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        associated_token::mint = vault_supported_token_mint,
        associated_token::authority = fund_reserve_account,
        associated_token::token_program = Token::id(),
    )]
    pub fund_vault_supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        associated_token::mint = vault_supported_token_mint,
        associated_token::authority = vault_account,
        associated_token::token_program = Token::id(),
    )]
    pub vault_vault_supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
}
