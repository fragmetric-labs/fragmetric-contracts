use anchor_lang::prelude::*;
use anchor_spl::{token_2022::Token2022, token_interface::{Mint, TokenAccount}};

use crate::errors::ErrorCode;
use crate::modules::common::*;
use crate::modules::fund::{FundAccount, ReceiptTokenLockAuthority, WithdrawalStatus};

pub struct FundWithdrawalJob<'a, 'info: 'a> {
    receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    receipt_token_program: &'a Program<'info, Token2022>,
    receipt_token_lock_authority: &'a mut Account<'info, ReceiptTokenLockAuthority>,
    receipt_token_lock_account: &'a mut InterfaceAccount<'info, TokenAccount>,
    fund_account: &'a mut Account<'info, FundAccount>,
    pricing_sources: &'a [AccountInfo<'info>],
}

impl<'a, 'info: 'a> FundWithdrawalJob<'a, 'info> {
    pub fn check_threshold(withdrawal_status: &'a WithdrawalStatus) -> Result<()> {
        let current_time = crate::utils::timestamp_now()?;

        let mut threshold_satisfied = matches!(
                withdrawal_status.last_batch_processing_started_at,
                Some(x) if (current_time - x) > withdrawal_status.batch_processing_threshold_duration
            );

        if withdrawal_status.pending_batch_withdrawal .receipt_token_to_process > withdrawal_status.batch_processing_threshold_amount {
            threshold_satisfied = true;
        }

        if !threshold_satisfied {
            err!(ErrorCode::OperatorJobUnmetThresholdError)?
        }
        Ok(())
    }

    pub fn new(
        receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        receipt_token_program: &'a Program<'info, Token2022>,
        receipt_token_lock_authority: &'a mut Account<'info, ReceiptTokenLockAuthority>,
        receipt_token_lock_account: &'a mut InterfaceAccount<'info, TokenAccount>,
        fund_account: &'a mut Account<'info, FundAccount>,
        pricing_sources: &'a [AccountInfo<'info>],
    ) -> Self {
        Self {
            receipt_token_mint,
            receipt_token_program,
            receipt_token_lock_authority,
            receipt_token_lock_account,
            fund_account,
            pricing_sources,
        }
    }

    pub fn process(&mut self) -> Result<(u64, u64)> {
        let fund_account = &mut self.fund_account;

        fund_account.withdrawal_status
            .start_processing_pending_batch_withdrawal()?;
        fund_account.update_token_prices(self.pricing_sources)?;

        let total_sol_value_in_fund = fund_account.assets_total_sol_value()?;
        let receipt_token_total_supply = self.receipt_token_mint.supply;
        let receipt_token_price =
            fund_account.receipt_token_sol_value_per_token(self.receipt_token_mint.decimals, receipt_token_total_supply)?;

        let mut receipt_token_amount_to_burn: u64 = 0;
        for batch in &mut fund_account.withdrawal_status.batch_withdrawals_in_progress {
            let amount = batch.receipt_token_to_process;
            batch.record_unstaking_start(amount)?;
            receipt_token_amount_to_burn = receipt_token_amount_to_burn
                .checked_add(amount)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        }

        let mut receipt_token_amount_not_burned = receipt_token_amount_to_burn;
        let mut total_sol_reserved_amount: u64 = 0;
        for batch in &mut fund_account.withdrawal_status.batch_withdrawals_in_progress {
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
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
            total_sol_reserved_amount = total_sol_reserved_amount
                .checked_add(sol_reserved_amount)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
            batch.record_unstaking_end(receipt_token_amount, sol_reserved_amount)?;
        }
        fund_account.sol_operation_reserved_amount = fund_account
            .sol_operation_reserved_amount
            .checked_sub(total_sol_reserved_amount)
            .ok_or_else(|| error!(ErrorCode::FundOperationReservedSOLExhaustedException))?;

        self.call_burn_token_cpi(receipt_token_amount_to_burn)?;
        let fund = &mut self.fund_account;

        fund.withdrawal_status
            .end_processing_completed_batch_withdrawals()?;

        Ok((receipt_token_price, receipt_token_total_supply))
    }

    fn call_burn_token_cpi(&mut self, amount: u64) -> Result<()> {
        self.receipt_token_program.burn_token_cpi(
            self.receipt_token_mint,
            self.receipt_token_lock_account,
            self.receipt_token_lock_authority.to_account_info(),
            Some(&[self.receipt_token_lock_authority.signer_seeds().as_ref()]),
            amount,
        )
    }
}
