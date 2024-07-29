use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::{mint_to, MintTo, Token2022};
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::{constants::*, Empty};

#[derive(Accounts)]
pub struct TokenMintReceiptToken<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    /// CHECK: receipt token account owner could be user or pda
    receipt_token_account_owner: UncheckedAccount<'info>,

    #[account(mut, address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [FUND_TOKEN_AUTHORITY_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub fund_token_authority: Account<'info, Empty>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = receipt_token_mint,
        associated_token::authority = receipt_token_account_owner,
        associated_token::token_program = token_program,
    )]
    pub receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>, // user's fragSOL token account

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> TokenMintReceiptToken<'info> {
    pub fn mint_receipt_token_for_test(ctx: Context<Self>, amount: u64) -> Result<()> {
        let receipt_token_account_key = ctx.accounts.receipt_token_account.key();
        msg!(
            "user's receipt_token_account key: {:?}",
            receipt_token_account_key
        );

        Self::mint_token_cpi(&ctx, amount)?;
        msg!(
            "Minted {} to user token account {:?}",
            amount,
            receipt_token_account_key
        );

        Ok(())
    }

    fn mint_token_cpi(ctx: &Context<Self>, amount: u64) -> Result<()> {
        let bump = ctx.bumps.fund_token_authority;
        // PDA signer seeds
        let receipt_token_mint_key = ctx.accounts.receipt_token_mint.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            FUND_TOKEN_AUTHORITY_SEED,
            receipt_token_mint_key.as_ref(),
            &[bump],
        ]];

        let mint_token_cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.receipt_token_mint.to_account_info(),
                to: ctx.accounts.receipt_token_account.to_account_info(),
                authority: ctx.accounts.fund_token_authority.to_account_info(),
            },
        )
        .with_signer(signer_seeds);

        mint_to(mint_token_cpi_ctx, amount)
    }
}
