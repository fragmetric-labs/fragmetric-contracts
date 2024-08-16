use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

use crate::{constants::*, fund::*, Empty};

#[derive(Accounts)]
pub struct FundInitialize<'info> {
    #[account(mut, address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        seeds = [FUND_SEED, receipt_token_mint.key().as_ref()], // fund + <any receipt token mint account>
        bump,
        space = 8 + Fund::INIT_SPACE,
    )]
    pub fund: Box<Account<'info, Fund>>,

    #[account(
        init,
        payer = admin,
        seeds = [FUND_TOKEN_AUTHORITY_SEED, receipt_token_mint.key().as_ref()],
        bump,
        space = 8 + Empty::INIT_SPACE,
    )]
    pub fund_token_authority: Account<'info, Empty>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>, // fragSOL token mint account
    #[account(
        init,
        payer = admin,
        associated_token::mint = receipt_token_mint,
        associated_token::authority = fund_token_authority,
        associated_token::token_program = token_program,
    )]
    pub receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>, // fund's fragSOL lock account

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> FundInitialize<'info> {
    pub fn initialize_fund(ctx: Context<Self>) -> Result<()> {
        let fund_token_authority_key = ctx.accounts.fund_token_authority.key();
        let receipt_token_mint_key = ctx.accounts.receipt_token_mint.key();
        msg!("receipt_token_mint: {}", receipt_token_mint_key);
        msg!("fund_token_authority: {}", fund_token_authority_key,);

        ctx.accounts
            .fund
            .initialize(ctx.accounts.admin.key(), receipt_token_mint_key);

        Ok(())
    }
}
