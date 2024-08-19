use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

use crate::{common::*, constants::*, error::ErrorCode, fund::*, token::*};

#[derive(Accounts)]
pub struct OperatorRun<'info> {
    // Only the admin can run the operator manually.
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
    pub fund_token_authority: Account<'info, FundTokenAuthority>,

    #[account(mut, address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = receipt_token_mint,
        associated_token::authority = fund_token_authority,
        associated_token::token_program = token_program,
    )]
    pub receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>, // fund's fragSOL lock account

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> OperatorRun<'info> {
    /// Manually run the operator.
    /// This instruction is only available to ADMIN
    pub fn operator_run(mut ctx: Context<Self>) -> Result<()> {
        let withdrawal_status = &mut ctx.accounts.fund.withdrawal_status;

        withdrawal_status.start_processing_pending_batch_withdrawal()?;

        let mut receipt_token_amount_to_burn: u64 = 0;
        for batch in &mut withdrawal_status.batch_withdrawals_in_progress {
            let amount = batch.receipt_token_to_process;
            batch.record_unstaking_start(amount)?;
            receipt_token_amount_to_burn = receipt_token_amount_to_burn
                .checked_add(amount)
                .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
        }

        let unstaking_ratio = 1;

        let mut receipt_token_amount_not_burned = receipt_token_amount_to_burn;
        for batch in &mut withdrawal_status.batch_withdrawals_in_progress {
            if receipt_token_amount_not_burned == 0 {
                break;
            }

            let receipt_token_amount = std::cmp::min(
                receipt_token_amount_not_burned,
                batch.receipt_token_being_processed,
            );
            receipt_token_amount_not_burned -= receipt_token_amount; // guaranteed to be safe
            let sol_amount_reserved = u64::try_from(
                (receipt_token_amount as u128)
                    .checked_mul(unstaking_ratio)
                    .ok_or_else(|| error!(ErrorCode::CalculationFailure))?,
            )
            .map_err(|_| error!(ErrorCode::CalculationFailure))?;
            batch.record_unstaking_end(receipt_token_amount, sol_amount_reserved)?;
        }

        Self::call_burn_token_cpi(&mut ctx, receipt_token_amount_to_burn)?;
        Self::call_transfer_hook(&ctx, receipt_token_amount_to_burn)?;

        let fund = &mut ctx.accounts.fund;

        let sol_amount_moved = u64::try_from(
            unstaking_ratio
                .checked_mul(receipt_token_amount_to_burn as u128)
                .ok_or_else(|| error!(ErrorCode::CalculationFailure))?,
        )
        .map_err(|_| error!(ErrorCode::CalculationFailure))?;

        fund.sol_amount_in = fund
            .sol_amount_in
            .checked_sub(sol_amount_moved)
            .ok_or_else(|| error!(ErrorCode::FundWithdrawalRequestExceedsSOLAmountsInTemp))?;

        fund.withdrawal_status
            .end_processing_completed_batch_withdrawals()
    }

    fn call_burn_token_cpi(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts.token_program.burn_token_cpi(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.receipt_token_lock_account,
            ctx.accounts.fund_token_authority.to_account_info(),
            Some(&[ctx.accounts.fund_token_authority.signer_seeds().as_ref()]),
            amount,
        )
    }

    fn call_transfer_hook(ctx: &Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts.receipt_token_mint.transfer_hook(
            Some(&ctx.accounts.receipt_token_lock_account),
            None,
            &ctx.accounts.fund,
            amount,
        )
    }
}
