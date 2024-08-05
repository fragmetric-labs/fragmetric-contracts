use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{burn, Burn, Mint, TokenAccount},
};
use solana_program::{
    clock::Clock,
};

use crate::{constants::*, fund::*, error::ErrorCode, Empty};

#[derive(Accounts)]
pub struct OperatorRunIfNeeded<'info> {
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
    pub fn operator_run(ctx: Context<Self>) {
        
        // get below configuration value from fund
        let TODO_LAST_PROCESS_TIME = 1722499490;
        let TODO_FUND_AMOUNT_THRESHOLD_CONFIG = 50_000_000_000; // 50 SOL
        let TODO_FUND_DURATION_THRESHOLD_CONFIG = 4032000; // 2weeks
        // Threshhold
        let TODO_UNMET_THRESHHOLD = false;

        let current = Clock::get()?;

        // if last_process_time is more than TODO_FUND_DURATION_THRESHOLD_CONFIG ago
        if (current.unix_timestamp - TODO_LAST_PROCESS_TIME) > 4032000 {
            TODO_UNMET_THRESHHOLD = true;
        }

        let fund = ctx.accounts.fund.to_latest_version();

        if fund.pending_withdrawals.receipt_token_to_process > 0 {
            fund.start_processing_pending_batch_withdrawal();
        }

        let receipt_token_amount_to_burn = fund
            .withdrawals_in_progress
            .batch_withdrawal_queue
            .iter_mut()
            .map(|batch| {
            let amount = batch.receipt_token_to_process;
            batch.record_processing_start(amount as u64);
            amount
            }).sum();

        // if receipt_token_amount_to_burn exceeds TODO_FUND_AMOUNT_THRESHOLD_CONFIG fragSOL
        if receipt_token_amount_to_burn > TODO_FUND_AMOUNT_THRESHOLD_CONFIG {
            TODO_UNMET_THRESHHOLD = true;
        }

        // if no condition is met, 
        if TODO_UNMET_THRESHHOLD {
            return err!(ErrorCode::OperatorUnmetThreshold);
        }

        Self::burn_token_cpi(&ctx, receipt_token_amount_to_burn as u64)?;

        let fund = ctx.accounts.fund.to_latest_version();

        let unstaking_ratio = 1;

        let mut burned_receipt_token_amount = receipt_token_amount_to_burn;
        for batch in fund
            .withdrawals_in_progress
            .batch_withdrawal_queue
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
            batch.record_processing_end(receipt_token_amount, sol_reserved);
        }

        fund.end_processing_completed_batch_withdrawals();

        Ok(())
    }

    fn burn_token_cpi(ctx: &Context<Self>, amount: u64) -> Result<()> {
        let Self {
            fund_token_authority,
            receipt_token_mint,
            receipt_token_lock_account,
            token_programm,
            ..
        } = &*ctx.accounts;

        let bump = ctx.bumps.fund_token_authority;
        let receipt_token_mint_key = receipt_token_mint.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            FUND_TOKEN_AUTHORITY_SEED,
            receipt_token_mint_key.as_ref(),
            &[bump],
        ]];

        let burn_token_cpi_ctx = CpiContext::new_with_signer(
            token_program.to_account_info(),
            Burn {
                mint: receipt_token_mint.to_account_info(),
                from: receipt_token_lock_account.to_account_info(),
                authority: fund_token_authority,to_account_info(),
            },
            signer_seeds,
        );

        burn(burn_token_cpi_ctx, amount)
    }
}
