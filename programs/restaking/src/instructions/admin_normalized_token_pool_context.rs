use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::constants::*;
use crate::modules::normalize::{NormalizedTokenAuthority, NormalizedTokenPoolConfig};
use crate::utils::PDASeeds;

#[derive(Accounts)]
pub struct AdminNormalizedTokenAccountInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub normalized_token_program: Program<'info, Token>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub normalized_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [NormalizedTokenAuthority::SEED, receipt_token_mint.key().as_ref(), normalized_token_mint.key().as_ref()],
        bump = normalized_token_authority.get_bump(),
        has_one = receipt_token_mint,
        has_one = normalized_token_mint,
    )]
    pub normalized_token_authority: Account<'info, NormalizedTokenAuthority>,

    #[account(
        init,
        payer = payer,
        token::mint = normalized_token_mint,
        token::authority = normalized_token_authority,
        token::token_program = normalized_token_program,
        seeds = [NormalizedTokenAuthority::NORMALIZED_TOKEN_ACCOUNT_SEED, receipt_token_mint.key().as_ref(), normalized_token_mint.key().as_ref()],
        bump,
    )]
    pub normalized_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
}

#[derive(Accounts)]
pub struct AdminNormalizedTokenPoolConfigInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub normalized_token_program: Program<'info, Token>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub normalized_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [NormalizedTokenAuthority::SEED, receipt_token_mint.key().as_ref(), normalized_token_mint.key().as_ref()],
        bump = normalized_token_authority.get_bump(),
        has_one = receipt_token_mint,
        has_one = normalized_token_mint,
    )]
    pub normalized_token_authority: Account<'info, NormalizedTokenAuthority>,

    #[account(
        token::mint = normalized_token_mint,
        token::authority = normalized_token_authority,
        token::token_program = normalized_token_program,
        seeds = [NormalizedTokenAuthority::NORMALIZED_TOKEN_ACCOUNT_SEED, receipt_token_mint.key().as_ref(), normalized_token_mint.key().as_ref()],
        bump,
    )]
    pub normalized_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init,
        payer = payer,
        space = 8 + NormalizedTokenPoolConfig::INIT_SPACE,
        seeds = [NormalizedTokenPoolConfig::SEED, receipt_token_mint.key().as_ref(), normalized_token_mint.key().as_ref()],
        bump,
    )]
    pub normalized_token_pool_config: Box<Account<'info, NormalizedTokenPoolConfig>>,
}
