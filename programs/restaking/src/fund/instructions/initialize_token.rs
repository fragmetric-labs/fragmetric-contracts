use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{common::*, constants::*, fund::*};

#[derive(Accounts)]
pub struct FundInitializeToken<'info> {
    #[account(mut, address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub token_mint: Box<InterfaceAccount<'info, Mint>>, // lst token mint account
    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = admin,
        seeds = [SupportedTokenAuthority::SEED, receipt_token_mint.key().as_ref(), token_mint.key().as_ref()],
        bump,
        space = 8 + SupportedTokenAuthority::INIT_SPACE,
    )]
    pub supported_token_authority: Account<'info, SupportedTokenAuthority>,

    #[account(
        init_if_needed,
        payer = admin,
        token::mint = token_mint,
        token::authority = supported_token_authority,
        seeds = [FUND_TOKEN_ACCOUNT_SEED, token_mint.key().as_ref()],
        bump,
    )]
    pub fund_token_account: Box<InterfaceAccount<'info, TokenAccount>>, // fund's lst token account

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> FundInitializeToken<'info> {
    pub fn initialize(ctx: Context<Self>) -> Result<()> {
        let receipt_token_mint_key = ctx.accounts.receipt_token_mint.key();
        let token_mint_key = ctx.accounts.token_mint.key();

        ctx.accounts.supported_token_authority.initialize_if_needed(
            ctx.bumps.supported_token_authority,
            receipt_token_mint_key,
            token_mint_key,
        );

        Ok(())
    }
}
