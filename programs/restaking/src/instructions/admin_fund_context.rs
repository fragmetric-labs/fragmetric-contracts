use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::Token;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::modules::fund::FundAccount;
use crate::modules::normalization::NormalizedTokenPoolAccount;
use crate::utils::PDASeeds;

// will be used only once
#[derive(Accounts)]
pub struct AdminFundAccountInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(mut, address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub receipt_token_program: Program<'info, Token2022>,

    #[account(
        init,
        payer = payer,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump,
        space = 8 + FundAccount::INIT_SPACE,
    )]
    pub fund_account: Box<Account<'info, FundAccount>>,
}

// will be used only once
#[derive(Accounts)]
pub struct AdminFundReceiptTokenLockAccountInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.get_bump(),
        has_one = receipt_token_mint,
        constraint = fund_account.is_latest_version() @ ErrorCode::InvalidDataVersionError,
    )]
    pub fund_account: Box<Account<'info, FundAccount>>,

    pub receipt_token_program: Program<'info, Token2022>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = receipt_token_mint,
        associated_token::authority = fund_account,
        associated_token::token_program = receipt_token_program,
    )]
    pub receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct AdminFundNormalizedTokenAccountInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.get_bump(),
        has_one = receipt_token_mint,
        constraint = fund_account.is_latest_version() @ ErrorCode::InvalidDataVersionError,
    )]
    pub fund_account: Box<Account<'info, FundAccount>>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(address = NSOL_MINT_ADDRESS)]
    pub normalized_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub normalized_token_program: Program<'info, Token>,

    // TODO(v0.4): replace init_if_needed to init after v0.3 migration
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = normalized_token_mint,
        associated_token::authority = fund_account,
        associated_token::token_program = normalized_token_program,
    )]
    pub fund_normalized_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(
        mut,
        seeds = [NormalizedTokenPoolAccount::SEED, normalized_token_mint.key().as_ref()],
        bump = normalized_token_pool_account.get_bump(),
        has_one = normalized_token_mint,
        constraint = normalized_token_pool_account.is_latest_version() @ ErrorCode::InvalidDataVersionError,
    )]
    pub normalized_token_pool_account: Box<Account<'info, NormalizedTokenPoolAccount>>,
}

#[derive(Accounts)]
pub struct AdminFundJitoRestakingProtocolAccountInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.get_bump(),
        has_one = receipt_token_mint,
        constraint = fund_account.is_latest_version() @ ErrorCode::InvalidDataVersionError,
    )]
    pub fund_account: Box<Account<'info, FundAccount>>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: just need to validate vault state is owned by the vault program
    #[account(address = JITO_VAULT_PROGRAM_ID)]
    pub vault_program: UncheckedAccount<'info>,

    /// CHECK: will be validated by pricing service
    #[account(address = FRAGSOL_JITO_VAULT_ACCOUNT_ADDRESS)]
    pub vault: UncheckedAccount<'info>,

    #[account(address = FRAGSOL_JITO_VAULT_RECEIPT_TOKEN_MINT_ADDRESS)]
    pub vault_receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(address = anchor_spl::token::ID)]
    pub vault_receipt_token_program: Interface<'info, TokenInterface>,

    #[account(address = NSOL_MINT_ADDRESS)]
    pub vault_supported_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(address = anchor_spl::token::ID)]
    pub vault_supported_token_program: Interface<'info, TokenInterface>,

    // TODO(v0.4): replace init_if_needed to init after v0.3 migration
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = vault_receipt_token_mint,
        associated_token::authority = fund_account,
        associated_token::token_program = vault_receipt_token_program,
    )]
    pub fund_vault_receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    // TODO(v0.4): replace init_if_needed to init after v0.3 migration
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = vault_supported_token_mint,
        associated_token::authority = fund_account,
        associated_token::token_program = vault_supported_token_program,
    )]
    pub fund_vault_supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct AdminFundAccountUpdateContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        realloc = 8 + FundAccount::INIT_SPACE,
        realloc::payer = payer,
        realloc::zero = false,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.get_bump(),
        has_one = receipt_token_mint,
    )]
    pub fund_account: Box<Account<'info, FundAccount>>,
}
