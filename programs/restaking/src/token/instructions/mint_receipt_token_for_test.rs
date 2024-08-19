use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::{common::*, constants::*, fund::*, token::*};

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
        seeds = [FundTokenAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_token_authority.bump,
        has_one = receipt_token_mint,
    )]
    pub fund_token_authority: Account<'info, FundTokenAuthority>,

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
    pub fn mint_receipt_token_for_test(mut ctx: Context<Self>, amount: u64) -> Result<()> {
        let receipt_token_account_key = ctx.accounts.receipt_token_account.key();
        msg!(
            "user's receipt_token_account key: {:?}",
            receipt_token_account_key
        );

        Self::call_mint_token_cpi(&mut ctx, amount)?;
        msg!(
            "Minted {} to user token account {:?}",
            amount,
            receipt_token_account_key
        );

        Ok(())
    }

    fn call_mint_token_cpi(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts.token_program.mint_token_cpi(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.receipt_token_account,
            ctx.accounts.fund_token_authority.to_account_info(),
            Some(&[ctx.accounts.fund_token_authority.signer_seeds().as_ref()]),
            amount,
        )
    }
}
