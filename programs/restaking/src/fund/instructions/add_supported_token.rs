use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

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
        mut,
        seeds = [FundTokenAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_token_authority.bump,
        has_one = receipt_token_mint,
    )]
    pub fund_token_authority: Box<Account<'info, FundTokenAuthority>>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut)]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = token_mint,
        associated_token::authority = fund_token_authority,
        associated_token::token_program = token_program,
    )]
    pub fund_token_account: Box<InterfaceAccount<'info, TokenAccount>>, // fund's lst token account

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> FundAddSupportedToken<'info> {
    pub fn add_supported_token(
        ctx: Context<Self>,
        token_cap: u64,
        pricing_source: PricingSource,
    ) -> Result<()> {
        let token = ctx.accounts.token_mint.key();
        ctx.accounts.fund.check_token_does_not_exist(&token)?;
        ctx.accounts
            .fund
            .add_supported_token(token, token_cap, pricing_source);

        Ok(())
    }
}
