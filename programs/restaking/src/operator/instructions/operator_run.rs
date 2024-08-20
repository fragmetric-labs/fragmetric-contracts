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

    // TODO: use address lookup table!
    // TODO: rename properly!
    // TODO: use address constraint!
    /// CHECK: will be checked and deserialized when needed
    pub pricing_source0: UncheckedAccount<'info>,

    // TODO: use address lookup table!
    // TODO: rename properly!
    // TODO: use address constraint!
    /// CHECK: will be checked and deserialized when needed
    pub pricing_source1: UncheckedAccount<'info>,

    // TODO: use address lookup table!
    // TODO: rename properly!
    // TODO: use address constraint!
    /// CHECK: will be checked and deserialized when needed
    pub pricing_source2: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> OperatorRun<'info> {
    /// Manually run the operator.
    /// This instruction is only available to ADMIN
    pub fn operator_run(mut ctx: Context<Self>) -> Result<()> {
        let withdrawal_status = &mut ctx.accounts.fund.withdrawal_status;

        withdrawal_status.start_processing_pending_batch_withdrawal()?;

        let fund = &mut ctx.accounts.fund;
        let sources = [
            ctx.accounts.pricing_source0.as_ref(),
            ctx.accounts.pricing_source1.as_ref(),
            ctx.accounts.pricing_source2.as_ref(),
        ];
        fund.update_token_prices(&sources)?;
        let total_sol_value_in_fund = fund.total_sol_value()?;
        let receipt_token_total_supply = ctx.accounts.receipt_token_mint.supply;

        let mut receipt_token_amount_to_burn: u64 = 0;
        for batch in &mut fund.withdrawal_status.batch_withdrawals_in_progress {
            let amount = batch.receipt_token_to_process;
            batch.record_unstaking_start(amount)?;
            receipt_token_amount_to_burn = receipt_token_amount_to_burn
                .checked_add(amount)
                .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
        }

        let mut receipt_token_amount_not_burned = receipt_token_amount_to_burn;
        let mut total_sol_reserved_amount: u64 = 0;
        for batch in &mut fund.withdrawal_status.batch_withdrawals_in_progress {
            if receipt_token_amount_not_burned == 0 {
                break;
            }

            let receipt_token_amount = std::cmp::min(
                receipt_token_amount_not_burned,
                batch.receipt_token_being_processed,
            );
            receipt_token_amount_not_burned -= receipt_token_amount; // guaranteed to be safe

            let sol_reserved_amount = crate::utils::proportional_amount(
                receipt_token_amount,
                total_sol_value_in_fund,
                receipt_token_total_supply,
            )
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
            total_sol_reserved_amount = total_sol_reserved_amount
                .checked_add(sol_reserved_amount)
                .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
            batch.record_unstaking_end(receipt_token_amount, sol_reserved_amount)?;
        }
        fund.sol_amount_in = fund
            .sol_amount_in
            .checked_sub(total_sol_reserved_amount)
            .ok_or_else(|| error!(ErrorCode::FundWithdrawalRequestExceedsSOLAmountsInTemp))?;

        Self::call_burn_token_cpi(&mut ctx, receipt_token_amount_to_burn)?;
        Self::call_transfer_hook(&ctx, receipt_token_amount_to_burn)?;

        ctx.accounts
            .fund
            .withdrawal_status
            .end_processing_completed_batch_withdrawals()
    }

    fn call_burn_token_cpi(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts.token_program.burn_token_cpi(
            &mut ctx.accounts.receipt_token_mint,
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
