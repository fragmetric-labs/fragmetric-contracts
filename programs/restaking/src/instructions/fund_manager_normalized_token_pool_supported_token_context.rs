use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::modules::fund::FundAccount;
use crate::modules::normalize::NormalizedTokenPoolAccount;
use crate::utils::PDASeeds;

#[derive(Accounts)]
pub struct FundManagerSupportedTokenLockAccountInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.get_bump(),
        has_one = receipt_token_mint,
        constraint = fund_account.is_latest_version() @ ErrorCode::InvalidDataVersionError,
    )]
    // TODO fund must have authority to configure normalized token pool - for now just fix normalized token mint address
    pub fund_account: Box<Account<'info, FundAccount>>,

    #[account(address = NSOL_MINT_ADDRESS)]
    pub normalized_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
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

    pub normalized_token_program: Program<'info, Token>,

    pub supported_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub supported_token_program: Interface<'info, TokenInterface>,

    #[account(
        init,
        payer = payer,
        token::mint = supported_token_mint,
        token::authority = fund_manager,
        token::token_program = supported_token_program,
        seeds = [
            NormalizedTokenPoolAccount::SUPPORTED_TOKEN_LOCK_ACCOUNT_SEED,
            normalized_token_mint.key().as_ref(),
            supported_token_mint.key().as_ref()
        ],
        bump,
    )]
    pub supported_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>,
}

#[derive(Accounts)]
pub struct FundManagerNormalizedTokenPoolSupportedTokenContext<'info> {
    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

    #[account(address = NSOL_MINT_ADDRESS)]
    pub normalized_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [NormalizedTokenPoolAccount::SEED, normalized_token_mint.key().as_ref()],
        bump = normalized_token_pool_account.get_bump(),
        has_one = normalized_token_mint,
    )]
    pub normalized_token_pool_account: Box<Account<'info, NormalizedTokenPoolAccount>>,

    pub supported_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub supported_token_program: Interface<'info, TokenInterface>,

    #[account(
        token::mint = supported_token_mint,
        token::authority = fund_manager,
        token::token_program = supported_token_program,
        seeds = [
            NormalizedTokenPoolAccount::SUPPORTED_TOKEN_LOCK_ACCOUNT_SEED,
            normalized_token_mint.key().as_ref(),
            supported_token_mint.key().as_ref()
        ],
        bump,
    )]
    pub supported_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>,
}
