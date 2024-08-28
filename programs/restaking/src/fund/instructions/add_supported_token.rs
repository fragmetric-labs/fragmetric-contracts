use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{common::*, constants::*, fund::*};

#[derive(Accounts)]
pub struct FundAddSupportedToken<'info> {
    #[account(mut, address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [Fund::SEED, receipt_token_mint.key().as_ref()],
        bump = fund.bump,
        has_one = receipt_token_mint,
    )]
    pub fund: Box<Account<'info, Fund>>,

    #[account(
        seeds = [SupportedTokenAuthority::SEED, receipt_token_mint.key().as_ref(), supported_token_mint.key().as_ref()],
        bump = supported_token_authority.bump,
        has_one = receipt_token_mint,
        has_one = supported_token_mint,
    )]
    pub supported_token_authority: Box<Account<'info, SupportedTokenAuthority>>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub supported_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        token::mint = supported_token_mint,
        token::authority = supported_token_authority,
        seeds = [FUND_SUPPORTED_TOKEN_ACCOUNT_SEED, supported_token_mint.key().as_ref()],
        bump,
    )]
    pub fund_supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>, // fund's lst token account

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> FundAddSupportedToken<'info> {
    pub fn add_supported_token(
        ctx: Context<Self>,
        capacity_amount: u64,
        pricing_source: TokenPricingSource,
    ) -> Result<()> {
        let mint = ctx.accounts.supported_token_mint.key();
        let decimals = ctx.accounts.supported_token_mint.decimals;
        ctx.accounts.fund.check_token_does_not_exist(&mint)?;
        ctx.accounts
            .fund
            .add_supported_token(mint, decimals, capacity_amount, pricing_source);

        Ok(())
    }
}
