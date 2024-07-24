use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::constants::*;
use crate::{error::ErrorCode, fund::*};

#[derive(Accounts)]
pub struct DepositSOL<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    #[account(
        mut,
        seeds = [FUND_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
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

impl<'info> DepositSOL<'info> {
    pub fn handler(ctx: Context<Self>, amount: u64) -> Result<()> {
        let receipt_token_account = ctx.accounts.receipt_token_account.to_account_info().key;
        msg!("receipt_token_account: {}", receipt_token_account);

        Self::transfer_sol_cpi_ctx(&ctx, amount)?;

        let fund = &mut ctx.accounts.fund;
        fund.deposit_sol(amount)
    }

    pub fn transfer_sol_cpi_ctx(ctx: &Context<Self>, amount: u64) -> Result<()> {
        let depositor = &ctx.accounts.depositor;
        let fund = &ctx.accounts.fund;

        let sol_transfer_cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: depositor.to_account_info(),
                to: fund.to_account_info(),
            },
        );

        msg!("Transferring from {} to {}", depositor.key, fund.key());

        let res = system_program::transfer(sol_transfer_cpi_ctx, amount);
        msg!("Transferred {} SOL", amount);

        if res.is_ok() {
            Ok(())
        } else {
            err!(ErrorCode::FundSolTransferFailed)
        }
    }
}
