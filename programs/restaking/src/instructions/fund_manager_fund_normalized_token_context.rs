use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::modules::fund::FundAccount;
use crate::modules::normalization::NormalizedTokenPoolAccount;
use crate::utils::PDASeeds;

#[derive(Accounts)]
pub struct FundManagerFundNormalizedTokenInitialContext<'info> {
    #[account(address = FUND_MANAGER_PUBKEY)]
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

    #[account(address = FRAGSOL_NORMALIZED_TOKEN_MINT_ADDRESS)]
    pub normalized_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub normalized_token_program: Program<'info, Token>,

    #[account(
        associated_token::mint = normalized_token_mint,
        associated_token::authority = fund_account,
        associated_token::token_program = normalized_token_program,
    )]
    pub fund_normalized_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [NormalizedTokenPoolAccount::SEED, normalized_token_mint.key().as_ref()],
        bump = normalized_token_pool_account.get_bump(),
        has_one = normalized_token_mint,
        constraint = normalized_token_pool_account.is_latest_version() @ ErrorCode::InvalidDataVersionError,
    )]
    pub normalized_token_pool_account: Box<Account<'info, NormalizedTokenPoolAccount>>,
}
