use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

use crate::{constants::*, error::ErrorCode, fund::*, token::*, Empty};
use fragmetric_util::Upgradable;

#[derive(Accounts)]
pub struct OperatorRun<'info> {
    // Only the admin can run the operator manually.
    #[account(mut, address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [FUND_SEED, receipt_token_mint.key().as_ref()],
        bump,
        realloc = 8 + Fund::INIT_SPACE,
        // TODO must paid by fund
        realloc::payer = admin,
        realloc::zero = false,
    )]
    pub fund: Account<'info, Fund>,

    #[account(
        mut,
        seeds = [FUND_TOKEN_AUTHORITY_SEED, receipt_token_mint.key().as_ref()],
        bump,)]
    pub fund_token_authority: Account<'info, Empty>,

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
    pub system_program: Program<'info, System>,
}

impl<'info> OperatorRun<'info> {
    /// Manually run the operator.
    /// This instruction is only available to ADMIN
    pub fn operator_run(mut ctx: Context<Self>) -> Result<()> {
        let fund = ctx.accounts.fund.to_latest_version();

        fund.withdrawal_status
            .start_processing_pending_batch_withdrawal()?;

        let mut receipt_token_amount_to_burn: u64 = 0;
        for batch in fund
            .withdrawal_status
            .batch_withdrawals_in_progress
            .iter_mut()
        {
            let amount = u64::try_from(batch.receipt_token_to_process)
                .map_err(|_| error!(ErrorCode::CalculationFailure))?;
            batch.record_unstaking_start(amount)?;
            receipt_token_amount_to_burn = receipt_token_amount_to_burn
                .checked_add(amount)
                .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
        }

        let unstaking_ratio = 1;

        let mut burned_receipt_token_amount = receipt_token_amount_to_burn;
        for batch in fund
            .withdrawal_status
            .batch_withdrawals_in_progress
            .iter_mut()
        {
            if burned_receipt_token_amount == 0 {
                break;
            }

            let receipt_token_amount = std::cmp::min(
                burned_receipt_token_amount as u128,
                batch.receipt_token_being_processed,
            ) as u64;

            burned_receipt_token_amount -= receipt_token_amount;
            let sol_reserved = u64::try_from(
                (receipt_token_amount as u128)
                    .checked_mul(unstaking_ratio)
                    .ok_or_else(|| error!(ErrorCode::CalculationFailure))?,
            )
            .map_err(|_| error!(ErrorCode::CalculationFailure))?;
            batch.record_unstaking_end(receipt_token_amount, sol_reserved)?;
        }

        Self::call_burn_token_cpi(&mut ctx, receipt_token_amount_to_burn)?;

        let fund = ctx.accounts.fund.to_latest_version();

        let sol_amount_moved = u64::try_from(
            (receipt_token_amount_to_burn as u128)
                .checked_mul(unstaking_ratio)
                .ok_or_else(|| error!(ErrorCode::CalculationFailure))?,
        )
        .map_err(|_| error!(ErrorCode::CalculationFailure))?;

        fund.sol_amount_in = fund
            .sol_amount_in
            .checked_sub(sol_amount_moved as u128)
            .ok_or_else(|| error!(ErrorCode::FundWithdrawalRequestExceedsSOLAmountsInTemp))?;

        fund.withdrawal_status
            .end_processing_completed_batch_withdrawals()
    }

    fn call_burn_token_cpi(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        let bump = ctx.bumps.fund_token_authority;
        let key = ctx.accounts.receipt_token_mint.key();
        let signer_seeds = [FUND_TOKEN_AUTHORITY_SEED, key.as_ref(), &[bump]];

        ctx.accounts.token_program.burn_token_cpi(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.receipt_token_lock_account,
            ctx.accounts.fund_token_authority.to_account_info(),
            Some(&[signer_seeds.as_ref()]),
            amount,
        )
    }
}
