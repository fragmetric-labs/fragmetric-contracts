use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::*;
use crate::modules::normalize::*;
use crate::utils::PDASeeds;

#[derive(Accounts)]
pub struct FundManagerNormalizedTokenPoolSupportedTokenLockAccountInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_amagner: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub supported_token_program: Interface<'info, TokenInterface>,

    pub normalized_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub supported_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [NormalizedTokenAuthority::SEED, normalized_token_mint.key().as_ref()],
        bump = normalized_token_authority.get_bump(),
        has_one = normalized_token_mint,
    )]
    pub normalized_token_authority: Account<'info, NormalizedTokenAuthority>,

    #[account(
        init,
        payer = payer,
        token::mint = supported_token_mint,
        token::authority = normalized_token_authority,
        token::token_program = supported_token_program,
        seeds = [
            NormalizedTokenAuthority::SUPPORTED_TOKEN_LOCK_ACCOUNT_SEED,
            normalized_token_mint.key().as_ref(),
            supported_token_mint.key().as_ref()
        ],
        bump,
    )]
    pub supported_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>,
}

#[derive(Accounts)]
pub struct FundManagerNormalizedTokenPoolSupportedTokenContext<'info> {
    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub normalized_token_program: Program<'info, Token>,

    pub supported_token_program: Interface<'info, TokenInterface>,

    pub normalized_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub supported_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [NormalizedTokenAuthority::SEED, normalized_token_mint.key().as_ref()],
        bump = normalized_token_authority.get_bump(),
        has_one = normalized_token_mint,
    )]
    pub normalized_token_authority: Account<'info, NormalizedTokenAuthority>,

    #[account(
        token::mint = normalized_token_mint,
        token::authority = normalized_token_authority,
        token::token_program = normalized_token_program,
        seeds = [NormalizedTokenAuthority::NORMALIZED_TOKEN_ACCOUNT_SEED, normalized_token_mint.key().as_ref()],
        bump,
    )]
    pub normalized_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [NormalizedTokenPoolConfig::SEED, normalized_token_mint.key().as_ref()],
        bump,
        has_one = normalized_token_mint,
        has_one = normalized_token_authority,
    )]
    pub normalized_token_pool_config: Box<Account<'info, NormalizedTokenPoolConfig>>,
}
