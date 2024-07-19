use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::fund::*;

#[derive(Accounts)]
pub struct DepositLST<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    #[account(mut)]
    pub fund: Account<'info, Fund>,

    #[account(mut)]
    pub token_mint: InterfaceAccount<'info, Mint>, // lst token mint account
    #[account(mut, token::mint = token_mint, token::authority = depositor.key())]
    pub depositor_token_account: InterfaceAccount<'info, TokenAccount>, // depositor's lst token account
    #[account(
        init,
        payer = depositor,
        seeds = [b"fund", token_mint.key().as_ref()],
        bump,
        token::mint = token_mint,
        token::authority = fund,
    )]
    pub fund_token_account: InterfaceAccount<'info, TokenAccount>, // fund's lst token account

    #[account(mut)]
    pub receipt_token_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = depositor,
        associated_token::mint = receipt_token_mint,
        associated_token::authority = depositor,
    )]
    pub receipt_token_account: InterfaceAccount<'info, TokenAccount>, // user's fragSOL token account

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<DepositLST>, amount: u64) -> Result<()> {
    let fund = &mut ctx.accounts.fund;

    Ok((deposit_lst(amount))?)
}
