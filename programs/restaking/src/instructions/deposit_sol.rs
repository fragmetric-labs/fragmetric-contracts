use anchor_lang::{prelude::*, system_program};
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::{fund::*, restaking};

#[derive(Accounts)]
pub struct DepositSOL<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    #[account(mut)]
    pub fund: Account<'info, Fund>,

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

pub fn handler(ctx: Context<DepositSOL>, amount: u64) -> Result<()> {
    let fund = &mut ctx.accounts.fund;
    let depositor = &mut ctx.accounts.depositor;

    msg!("depositor {}, fund {}", depositor.key, fund.key());

    let sol_transfer_cpi_ctx = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
            from: depositor.to_account_info(),
            to: fund.to_account_info(),
        }
    );

    msg!("Transferring from {} to {}", depositor.key, fund.key());

    let res = system_program::transfer(sol_transfer_cpi_ctx, amount);

    msg!("Transferred {} SOL", amount);

    if res.is_ok() {
        Ok(())
    } else {
        err!(ErrorCode::TransferFailed)
    }

    // Ok(fund.deposit_sol(amount))
}

#[error_code]
pub enum ErrorCode {
    #[msg("Sol Transfer Failed")]
    TransferFailed,
}
