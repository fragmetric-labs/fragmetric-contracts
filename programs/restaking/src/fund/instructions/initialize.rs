use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

use crate::{common::*, constants::*, fund::*};

#[derive(Accounts)]
pub struct FundInitialize<'info> {
    #[account(mut, address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(
        init_if_needed,
        payer = admin,
        seeds = [Fund::SEED, receipt_token_mint.key().as_ref()], // fund + <any receipt token mint account>
        bump,
        space = 8 + Fund::INIT_SPACE,
    )]
    pub fund: Box<Account<'info, Fund>>,

    #[account(
        init_if_needed,
        payer = admin,
        seeds = [ReceiptTokenLockAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump,
        space = 8 + ReceiptTokenLockAuthority::INIT_SPACE,
    )]
    pub receipt_token_lock_authority: Account<'info, ReceiptTokenLockAuthority>,

    #[account(
        init_if_needed,
        payer = admin,
        seeds = [ReceiptTokenMintAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump,
        space = 8 + ReceiptTokenMintAuthority::INIT_SPACE,
    )]
    pub receipt_token_mint_authority: Account<'info, ReceiptTokenMintAuthority>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>, // fragSOL token mint account

    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = receipt_token_mint,
        associated_token::authority = receipt_token_lock_authority,
        associated_token::token_program = token_program,
    )]
    pub receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>, // fund's fragSOL lock account

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> FundInitialize<'info> {
    pub fn initialize_fund(ctx: Context<Self>) -> Result<()> {
        let receipt_token_mint_key = ctx.accounts.receipt_token_mint.key();

        ctx.accounts
            .fund
            .initialize_if_needed(ctx.bumps.fund, receipt_token_mint_key);
        ctx.accounts
            .receipt_token_lock_authority
            .initialize_if_needed(ctx.bumps.receipt_token_lock_authority, receipt_token_mint_key);
        ctx.accounts
            .receipt_token_mint_authority
            .initialize_if_needed(ctx.bumps.receipt_token_mint_authority, receipt_token_mint_key);

        Ok(())
    }
}
