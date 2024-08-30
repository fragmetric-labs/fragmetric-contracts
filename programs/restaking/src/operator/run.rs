use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

use crate::{common::*, error::ErrorCode, fund::*, token::*};

pub(super) struct Run<'a, 'info: 'a> {
    fund: &'a mut Fund,
    receipt_token_lock_authority: &'a mut Account<'info, ReceiptTokenLockAuthority>,
    receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    receipt_token_lock_account: &'a mut InterfaceAccount<'info, TokenAccount>,
    pricing_sources: &'a [&'a AccountInfo<'info>],
    token_program: &'a Program<'info, Token2022>,
}

impl<'a, 'info: 'a> Run<'a, 'info> {
    pub(super) fn new(
        fund: &'a mut Fund,
        receipt_token_lock_authority: &'a mut Account<'info, ReceiptTokenLockAuthority>,
        receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        receipt_token_lock_account: &'a mut InterfaceAccount<'info, TokenAccount>,
        pricing_sources: &'a [&'a AccountInfo<'info>],
        token_program: &'a Program<'info, Token2022>,
    ) -> Self {
        Self {
            fund,
            receipt_token_lock_authority,
            receipt_token_mint,
            receipt_token_lock_account,
            pricing_sources,
            token_program,
        }
    }

    pub(super) fn run(&mut self) -> Result<(u64, u64)> {
        let fund = &mut self.fund;

        fund.withdrawal_status
            .start_processing_pending_batch_withdrawal()?;
        fund.update_token_prices(self.pricing_sources)?;
        let total_sol_value_in_fund = fund.total_sol_value()?;
        let receipt_token_total_supply = self.receipt_token_mint.supply;
        let receipt_token_price =
            fund.receipt_token_price(self.receipt_token_mint.decimals, receipt_token_total_supply)?;

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
        fund.sol_operation_reserved_amount = fund
            .sol_operation_reserved_amount
            .checked_sub(total_sol_reserved_amount)
            .ok_or_else(|| error!(ErrorCode::FundWithdrawalRequestExceedsSOLAmountsInTemp))?;

        self.call_burn_token_cpi(receipt_token_amount_to_burn)?;
        let fund = &mut self.fund;

        fund.withdrawal_status
            .end_processing_completed_batch_withdrawals()?;

        Ok((receipt_token_price, receipt_token_total_supply))
    }

    fn call_burn_token_cpi(&mut self, amount: u64) -> Result<()> {
        self.token_program.burn_token_cpi(
            self.receipt_token_mint,
            self.receipt_token_lock_account,
            self.receipt_token_lock_authority.to_account_info(),
            Some(&[self.receipt_token_lock_authority.signer_seeds().as_ref()]),
            amount,
        )
    }
}
