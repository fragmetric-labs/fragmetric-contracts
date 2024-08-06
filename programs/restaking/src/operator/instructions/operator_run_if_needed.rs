use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

use crate::{constants::*, error::ErrorCode, fund::*, token::*, Empty};
use fragmetric_util::Upgradable;

#[derive(Accounts)]
pub struct OperatorRunIfNeeded<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [FUND_SEED, receipt_token_mint.key().as_ref()],
        bump,
        realloc = 8 + Fund::INIT_SPACE,
        // TODO must paid by fund
        realloc::payer = payer,
        realloc::zero = false,
    )]
    pub fund: Account<'info, Fund>,

    #[account(
        mut,
        seeds = [FUND_TOKEN_AUTHORITY_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub fund_token_authority: Account<'info, Empty>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
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

impl<'info> OperatorRunIfNeeded<'info> {
    /// RUn operator if conditions are met.
    /// This instructions is available to anyone.
    /// However, the threshold should be met
    pub fn operator_run_if_needed(mut ctx: Context<Self>) -> Result<()> {
        let fund = ctx.accounts.fund.to_latest_version();

        // if last_process_time is more than TODO_FUND_DURATION_THRESHOLD_CONFIG ago
        let current_time = Clock::get()?.unix_timestamp;

        let mut threshold_satified = match fund.withdrawal_status.last_batch_processing_started_at {
            Some(x)
                if (current_time - x)
                    > fund.withdrawal_status.batch_processing_threshold_duration =>
            {
                true
            }
            _ => false,
        };

        if fund
            .withdrawal_status
            .pending_batch_withdrawal
            .receipt_token_to_process
            > fund.withdrawal_status.batch_processing_threshold_amount
        {
            threshold_satified = true;
        }

        if threshold_satified {
            return err!(ErrorCode::OperatorUnmetThreshold);
        }

        let receipt_token_amount_to_burn = fund
            .withdrawal_status
            .batch_withdrawals_in_progress
            .iter_mut()
            .map(|batch| {
                let amount = batch.receipt_token_to_process;
                batch.record_unstaking_start(amount as u64);
                amount
            })
            .sum();

        fund.withdrawal_status
            .start_processing_pending_batch_withdrawal()?;

        Self::call_burn_token_cpi(&mut ctx, receipt_token_amount_to_burn as u64)?;

        let fund = ctx.accounts.fund.to_latest_version();

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
                burned_receipt_token_amount,
                batch.receipt_token_being_processed,
            );

            burned_receipt_token_amount -= receipt_token_amount;
            let sol_reserved = receipt_token_amount * unstaking_ratio;
            batch.record_unstaking_end(receipt_token_amount as u64, sol_reserved as u64);
        }

        let sol_amount_moved = unstaking_ratio * receipt_token_amount_to_burn;

        fund.sol_amount_in = fund
            .sol_amount_in
            .checked_sub(sol_amount_moved)
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
