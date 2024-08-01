use anchor_lang::prelude::*;

use crate::{error::ErrorCode, fund::*};

impl BatchWithdrawal {
    fn add_withdrawal_request(&mut self, amount: u64) {
        self.num_withdrawal_requests += 1;
        self.receipt_token_to_process += amount as u128;
    }

    fn next_batch_id(&self) -> u64 {
        self.batch_id + 1
    }

    // Called by operator
    #[allow(unused)]
    pub(crate) fn record_processing_start(&mut self, amount: u64) {
        self.receipt_token_to_process -= amount as u128;
        self.receipt_token_being_processed += amount as u128;
    }

    // Called by operator
    #[allow(unused)]
    pub(crate) fn record_processing_end(&mut self, receipt_token_amount: u64, sol_amount: u64) {
        self.receipt_token_being_processed -= receipt_token_amount as u128;
        self.receipt_token_processed += receipt_token_amount as u128;
        self.sol_reserved += sol_amount as u128;
    }

    fn is_completed(&self) -> bool {
        self.receipt_token_to_process == 0 && self.receipt_token_being_processed == 0
    }
}

impl WithdrawalsInProgress {
    fn push_batch_in_progress(&mut self, batch: BatchWithdrawal) {
        self.num_withdrawal_requests_in_progress += batch.num_withdrawal_requests;
        self.batch_withdrawal_queue.push(batch);
    }

    fn pop_completed_batches(&mut self) -> Vec<BatchWithdrawal> {
        let (completed, remaining) = std::mem::take(&mut self.batch_withdrawal_queue)
            .into_iter()
            .partition(|batch| batch.is_completed());
        self.batch_withdrawal_queue = remaining;
        completed.iter().for_each(|batch| {
            self.num_withdrawal_requests_in_progress -= batch.num_withdrawal_requests
        });
        completed
    }
}

impl ReservedFund {
    fn record_completed_batch_withdrawal(&mut self, batch: BatchWithdrawal) {
        self.last_completed_batch_id = batch.batch_id;
        self.num_completed_withdrawal_requests += batch.num_withdrawal_requests;
        self.total_receipt_token_processed += batch.receipt_token_processed;
        self.total_sol_reserved += batch.sol_reserved;
        self.sol_remaining += batch.sol_reserved;
    }

    fn withdraw_sol(&mut self, batch_id: u64, amount: u64) -> Result<()> {
        if batch_id > self.last_completed_batch_id {
            err!(ErrorCode::FundWithdrawlNotCompleted)?
        }

        self.sol_remaining = self
            .sol_remaining
            .checked_sub(amount as u128)
            .ok_or_else(|| error!(ErrorCode::FundNotEnoughReservedSol))?;

        Ok(())
    }
}

impl FundV2 {
    pub(super) fn create_withdrawal_request(
        &mut self,
        receipt_token_amount: u64,
    ) -> Result<WithdrawalRequest> {
        self.check_is_withdrawal_enabled()?;

        self.pending_withdrawals
            .add_withdrawal_request(receipt_token_amount);
        WithdrawalRequest::new(
            self.pending_withdrawals.batch_id,
            self.current_request_id(),
            receipt_token_amount,
        )
    }

    fn current_request_id(&self) -> u64 {
        self.reserved_fund.num_completed_withdrawal_requests
            + self
                .withdrawals_in_progress
                .num_withdrawal_requests_in_progress
            + self.pending_withdrawals.num_withdrawal_requests
    }

    fn check_is_withdrawal_enabled(&self) -> Result<()> {
        if !self.withdrawal_enabled_flag {
            err!(ErrorCode::FundWithdrawalDisabled)?
        }

        Ok(())
    }

    pub(super) fn withdraw_sol(&mut self, batch_id: u64, amount: u64) -> Result<()> {
        self.check_is_withdrawal_enabled()?;
        self.reserved_fund.withdraw_sol(batch_id, amount)
    }

    // Called by operator
    #[allow(unused)]
    pub(crate) fn start_processing_pending_batch_withdrawal(&mut self) {
        let new = BatchWithdrawal::new(self.pending_withdrawals.next_batch_id());
        let old = std::mem::replace(&mut self.pending_withdrawals, new);
        self.withdrawals_in_progress.push_batch_in_progress(old);
    }

    // Called by operator
    #[allow(unused)]
    pub(crate) fn end_processing_completed_batch_withdrawals(&mut self) {
        self.withdrawals_in_progress
            .pop_completed_batches()
            .into_iter()
            .for_each(|batch| {
                self.reserved_fund.record_completed_batch_withdrawal(batch);
            });
    }
}

impl UserAccountV1 {
    pub(super) fn push_withdrawal_request(&mut self, request: WithdrawalRequest) -> Result<()> {
        // Check max withdrawal request amount (constant)??
        self.withdrawal_requests.push(request);

        Ok(())
    }

    pub(super) fn pop_withdrawal_request(&mut self, request_id: u64) -> Result<WithdrawalRequest> {
        let (target_requests, remaining_requests): (Vec<_>, _) =
            std::mem::take(&mut self.withdrawal_requests)
                .into_iter()
                .partition(|req| req.request_id == request_id);
        self.withdrawal_requests = remaining_requests;
        target_requests
            .into_iter()
            .next()
            .ok_or_else(|| error!(ErrorCode::FundWithdrawalRequestNotFound))
    }
}
