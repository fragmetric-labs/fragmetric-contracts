use anchor_lang::prelude::*;

use crate::{error::ErrorCode, fund::*};

impl BatchWithdrawal {
    fn add_withdrawal_request(&mut self, amount: u64) {
        self.num_withdrawal_requests += 1;
        self.receipt_token_to_process += amount as u128;
    }

    fn remove_withdrawal_request(&mut self, amount: u64) {
        self.num_withdrawal_requests -= 1;
        self.receipt_token_to_process -= amount as u128;
    }

    fn start_batch_processing(&mut self) -> Result<()> {
        self.processing_started_at = Some(Clock::get()?.unix_timestamp);
        Ok(())
    }

    fn is_completed(&self) -> bool {
        self.processing_started_at.is_some()
            && self.receipt_token_to_process == 0
            && self.receipt_token_being_processed == 0
    }

    // Called by operator
    pub(crate) fn record_unstaking_start(&mut self, receipt_token_amount: u64) {
        self.receipt_token_to_process -= receipt_token_amount as u128;
        self.receipt_token_being_processed += receipt_token_amount as u128;
    }

    // Called by operator
    pub(crate) fn record_unstaking_end(&mut self, receipt_token_amount: u64, sol_amount: u64) {
        self.receipt_token_being_processed -= receipt_token_amount as u128;
        self.receipt_token_processed += receipt_token_amount as u128;
        self.sol_reserved += sol_amount as u128;
    }
}

impl ReservedFund {
    fn record_completed_batch_withdrawal(&mut self, batch: BatchWithdrawal) {
        self.num_completed_withdrawal_requests += batch.num_withdrawal_requests;
        self.total_receipt_token_processed += batch.receipt_token_processed;
        self.total_sol_reserved += batch.sol_reserved;
        self.sol_remaining += batch.sol_reserved;
    }

    fn withdraw_sol(&mut self, amount: u64) -> Result<()> {
        self.sol_remaining = self
            .sol_remaining
            .checked_sub(amount as u128)
            .ok_or_else(|| error!(ErrorCode::FundNotEnoughReservedSol))?;

        Ok(())
    }
}

impl WithdrawalStatus {
    pub(super) fn create_withdrawal_request(
        &mut self,
        receipt_token_amount: u64,
    ) -> Result<WithdrawalRequest> {
        self.check_is_withdrawal_enabled()?;

        let request_id = self.next_request_id;
        self.next_request_id += 1;

        self.pending_batch_withdrawal
            .add_withdrawal_request(receipt_token_amount);
        WithdrawalRequest::new(
            self.pending_batch_withdrawal.batch_id,
            request_id,
            receipt_token_amount,
        )
    }

    pub(super) fn cancel_withdrawal_request(
        &mut self,
        batch_id: u64,
        receipt_token_amount: u64,
    ) -> Result<()> {
        self.check_is_batch_processing_not_started(batch_id)?;
        self.pending_batch_withdrawal
            .remove_withdrawal_request(receipt_token_amount);

        Ok(())
    }

    pub(super) fn withdraw_sol(&mut self, batch_id: u64, amount: u64) -> Result<u64> {
        self.check_is_withdrawal_enabled()?;
        self.check_is_batch_processing_completed(batch_id)?;
        let amount = self.deduct_withdrawal_fee(amount);
        self.reserved_fund.withdraw_sol(amount)?;

        Ok(amount)
    }

    fn deduct_withdrawal_fee(&self, amount: u64) -> u64 {
        amount - amount * self.sol_withdrawal_fee_rate as u64 / 10000
    }

    fn check_is_withdrawal_enabled(&self) -> Result<()> {
        if !self.withdrawal_enabled_flag {
            err!(ErrorCode::FundWithdrawalDisabled)?
        }

        Ok(())
    }

    fn check_is_batch_processing_not_started(&self, batch_id: u64) -> Result<()> {
        if batch_id < self.pending_batch_withdrawal.batch_id {
            err!(ErrorCode::FundWithdrawalAlreadyInProgress)?
        }

        Ok(())
    }

    fn check_is_batch_processing_completed(&self, batch_id: u64) -> Result<()> {
        if batch_id > self.last_completed_batch_id {
            err!(ErrorCode::FundWithdrawalNotCompleted)?
        }

        Ok(())
    }

    // Called by operator
    pub(crate) fn start_processing_pending_batch_withdrawal(&mut self) -> Result<()> {
        let batch_id = self.next_batch_id;
        self.next_batch_id += 1;
        let new = BatchWithdrawal::empty(batch_id);

        let mut old = std::mem::replace(&mut self.pending_batch_withdrawal, new);
        old.start_batch_processing()?;

        self.num_withdrawal_requests_in_progress += old.num_withdrawal_requests;
        self.last_batch_processing_started_at = old.processing_started_at;
        self.batch_withdrawals_in_progress.push(old);

        Ok(())
    }

    // Called by operator
    pub(crate) fn end_processing_completed_batch_withdrawals(&mut self) -> Result<()> {
        let completed_batch_withdrawals = self.pop_completed_batch_withdrawals();
        if let Some(batch) = completed_batch_withdrawals.last() {
            self.last_completed_batch_id = batch.batch_id;
            self.last_batch_processing_completed_at = Some(Clock::get()?.unix_timestamp);
        }
        completed_batch_withdrawals.into_iter().for_each(|batch| {
            self.reserved_fund.record_completed_batch_withdrawal(batch);
        });
        Ok(())
    }

    fn pop_completed_batch_withdrawals(&mut self) -> Vec<BatchWithdrawal> {
        let (completed, remaining) = std::mem::take(&mut self.batch_withdrawals_in_progress)
            .into_iter()
            .partition(|batch| {
                if batch.is_completed() {
                    self.num_withdrawal_requests_in_progress -= batch.num_withdrawal_requests;
                    true
                } else {
                    false
                }
            });
        self.batch_withdrawals_in_progress = remaining;
        completed
    }
}

impl UserReceipt {
    pub(super) fn push_withdrawal_request(&mut self, request: WithdrawalRequest) -> Result<()> {
        // Check max withdrawal request amount (constant)??
        self.withdrawal_requests.push(request);

        Ok(())
    }

    pub(super) fn pop_withdrawal_request(&mut self, request_id: u64) -> Result<WithdrawalRequest> {
        let index = self
            .withdrawal_requests
            .binary_search_by_key(&request_id, |req| req.request_id)
            .map_err(|_| error!(ErrorCode::FundWithdrawalRequestNotFound))?;
        Ok(self.withdrawal_requests.remove(index))
    }
}
