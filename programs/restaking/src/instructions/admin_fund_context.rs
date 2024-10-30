use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::modules::fund::{FundAccount, ReceiptTokenLockAuthority};
use crate::utils::PDASeeds;

// will be used only once
#[derive(Accounts)]
pub struct AdminFundReceiptTokenLockAuthorityInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = payer,
        seeds = [ReceiptTokenLockAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump,
        space = 8 + ReceiptTokenLockAuthority::INIT_SPACE,
    )]
    pub receipt_token_lock_authority: Account<'info, ReceiptTokenLockAuthority>,
}

// will be used only once
#[derive(Accounts)]
pub struct AdminFundReceiptTokenLockAccountInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub receipt_token_program: Program<'info, Token2022>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [ReceiptTokenLockAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump = receipt_token_lock_authority.get_bump(),
        has_one = receipt_token_mint,
    )]
    pub receipt_token_lock_authority: Account<'info, ReceiptTokenLockAuthority>,

    #[account(
        init,
        payer = payer,
        token::mint = receipt_token_mint,
        token::authority = receipt_token_lock_authority,
        token::token_program = receipt_token_program,
        seeds = [ReceiptTokenLockAuthority::TOKEN_ACCOUNT_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>,
}

// will be used only once
#[derive(Accounts)]
pub struct AdminFundAccountInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = payer,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump,
        space = 8 + FundAccount::INIT_SPACE,
    )]
    pub fund_account: Box<Account<'info, FundAccount>>,
}

#[derive(Accounts)]
pub struct AdminFundNormalizedTokenAccountInitialContext<'info> {
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

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(address = NSOL_MINT_ADDRESS)]
    pub normalized_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub normalized_token_program: Program<'info, Token>,

    #[account(
        init,
        payer = payer,
        token::mint = normalized_token_mint,
        token::authority = fund_account,
        token::token_program = normalized_token_program,
        seeds = [
            FundAccount::NORMALIZED_TOKEN_ACCOUNT_SEED,
            normalized_token_mint.key().as_ref()
        ],
        bump,
    )]
    pub fund_normalized_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
}

#[derive(Accounts)]
pub struct AdminFundAccountUpdateContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.get_bump(),
        has_one = receipt_token_mint,
    )]
    pub fund_account: Box<Account<'info, FundAccount>>,
}
