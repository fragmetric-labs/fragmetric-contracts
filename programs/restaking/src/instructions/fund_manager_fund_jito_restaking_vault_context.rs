use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::modules::fund::FundAccount;
use crate::utils::{AccountLoaderExt, PDASeeds};

#[event_cpi]
#[derive(Accounts)]
pub struct FundManagerFundJitoRestakingVaultInitialContext<'info> {
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

    /// CHECK: just need to validate vault state is owned by the vault program
    #[account(address = JITO_VAULT_PROGRAM_ID)]
    pub vault_program: UncheckedAccount<'info>,

    /// CHECK: will be validated by pricing service
    pub vault_account: UncheckedAccount<'info>,

    pub vault_receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(address = anchor_spl::token::ID)]
    pub vault_receipt_token_program: Interface<'info, TokenInterface>,

    pub vault_supported_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(address = anchor_spl::token::ID)]
    pub vault_supported_token_program: Interface<'info, TokenInterface>,

    #[account(
        associated_token::mint = vault_receipt_token_mint,
        associated_token::authority = fund_reserve_account,
        associated_token::token_program = vault_receipt_token_program,
    )]
    pub fund_vault_receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        associated_token::mint = vault_supported_token_mint,
        associated_token::authority = fund_reserve_account,
        associated_token::token_program = vault_supported_token_program,
    )]
    pub fund_vault_supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        associated_token::mint = vault_supported_token_mint,
        associated_token::authority = vault_account,
        associated_token::token_program = vault_supported_token_program,
    )]
    pub vault_vault_supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct FundManagerFundJitoRestakingVaultDelegationInitialContext<'info> {
    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = fund_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub fund_account: AccountLoader<'info, FundAccount>,

    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: will be validated by pricing service
    pub vault_account: UncheckedAccount<'info>,

    /// CHECK: just need to validate vault operator is owned by the jito restaking program
    #[account(address = JITO_RESTAKING_PROGRAM_ID)]
    pub jito_restaking_program: UncheckedAccount<'info>,

    /// CHECK: just verify that it's owner is Jito Restaking Program
    pub vault_operator: UncheckedAccount<'info>,
}
