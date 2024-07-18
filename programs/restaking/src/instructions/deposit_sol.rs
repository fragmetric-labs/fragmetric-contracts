use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::fund::*;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct DepositSOL<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    #[account(mut)]
    pub fund: Account<'info, Fund>,

    pub receipt_token_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = depositor,
        associated_token::mint = receipt_token_mint,
        associated_token::authority = depositor,
        associated_token::token_program = token_program,
    )]
    pub receipt_token_account: InterfaceAccount<'info, TokenAccount>, // user's fragSOL token account

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<DepositSOL>, amount: u64) -> Result<()> {
    let fund = &mut ctx.accounts.fund;
    let depositor = &mut ctx.accounts.depositor;
    let receipt_token_account = ctx.accounts.receipt_token_account.to_account_info().key;
    msg!("receipt_token_account: {}", receipt_token_account);

    let res = deposit_sol(
        depositor,
        fund,
        &ctx.accounts.system_program,
        amount
    );

    if res.is_ok() {
        Ok(())
    } else {
        err!(ErrorCode::SolTransferFailed)
    }
}
