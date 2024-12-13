use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::modules::fund::FundAccount;
use crate::utils::{AccountLoaderExt, PDASeeds};

#[derive(Accounts)]
pub struct FundManagerFundJitoRestakingVaultInitialContext<'info> {
    #[account(address = FUND_MANAGER_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = fund_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub fund_account: AccountLoader<'info, FundAccount>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: just need to validate vault state is owned by the vault program
    #[account(address = JITO_VAULT_PROGRAM_ID)]
    pub vault_program: UncheckedAccount<'info>,

    /// CHECK: will be validated by pricing service
    #[account(address = FRAGSOL_JITO_VAULT_ACCOUNT_ADDRESS)]
    pub vault_account: UncheckedAccount<'info>,

    #[account(address = FRAGSOL_JITO_VAULT_RECEIPT_TOKEN_MINT_ADDRESS)]
    pub vault_receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(address = anchor_spl::token::ID)]
    pub vault_receipt_token_program: Interface<'info, TokenInterface>,

    #[account(address = FRAGSOL_NORMALIZED_TOKEN_MINT_ADDRESS)]
    pub vault_supported_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(address = anchor_spl::token::ID)]
    pub vault_supported_token_program: Interface<'info, TokenInterface>,

    #[account(
        associated_token::mint = vault_receipt_token_mint,
        associated_token::authority = fund_account,
        associated_token::token_program = vault_receipt_token_program,
    )]
    pub fund_vault_receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        associated_token::mint = vault_supported_token_mint,
        associated_token::authority = fund_account,
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
