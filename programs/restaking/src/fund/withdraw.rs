use anchor_lang::prelude::*;

use crate::{error::ErrorCode, fund::*};

impl BatchWithdrawal {
    fn add_withdrawal_request(&mut self, amount: u64) -> Result<()> {
        // Check max withdrawal request amount (constant)??
        self.num_withdrawal_requests += 1;
        self.receipt_token_to_process += amount as u128;

        Ok(())
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

    fn pop_completed_batch(&mut self) -> Option<BatchWithdrawal> {
        let completed_batch = self
            .batch_withdrawal_queue
            .first()
            .filter(|batch| batch.is_completed())?;

        self.num_withdrawal_requests_in_progress -= completed_batch.num_withdrawal_requests;
        let mut batch_iter = std::mem::take(&mut self.batch_withdrawal_queue).into_iter();
        let completed_batch = batch_iter.next();
        self.batch_withdrawal_queue = batch_iter.collect();

        completed_batch
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

    fn check_if_withdrawal_completed(&self, batch_id: u64) -> Result<()> {
        if batch_id > self.last_completed_batch_id {
            err!(ErrorCode::FundWithdrawlNotCompleted)?
        }

        Ok(())
    }

    pub(super) fn withdraw_sol(&mut self, receipt_token_amount: u64) -> Result<u64> {
        // TODO later we have to use oracle data, now 1:1
        #[allow(clippy::identity_op)]
        let withdraw_amount = receipt_token_amount * 1;

        self.sol_remaining = self
            .sol_remaining
            .checked_sub(withdraw_amount as u128)
            .ok_or_else(|| error!(ErrorCode::FundNotEnoughReservedSol))?;

        Ok(withdraw_amount)
    }
}

impl FundV2 {
    pub(super) fn request_withdrawal(
        &mut self,
        receipt_token_amount: u64,
    ) -> Result<WithdrawalRequest> {
        self.pending_withdrawals
            .add_withdrawal_request(receipt_token_amount)?;
        WithdrawalRequest::new(self.pending_withdrawals.batch_id, self.current_request_id())
    }

    fn current_request_id(&self) -> u64 {
        self.reserved_fund.num_completed_withdrawal_requests
            + self
                .withdrawals_in_progress
                .num_withdrawal_requests_in_progress
            + self.pending_withdrawals.num_withdrawal_requests
    }

    // Called by operator
    #[allow(unused)]
    pub(crate) fn start_batch_withdrawal(&mut self) {
        let new = BatchWithdrawal::new(self.pending_withdrawals.next_batch_id());
        let old = std::mem::replace(&mut self.pending_withdrawals, new);
        self.withdrawals_in_progress.push_batch_in_progress(old);
    }

    // Called by operator
    #[allow(unused)]
    pub(crate) fn end_batch_withdrawal(&mut self) {
        if let Some(batch) = self.withdrawals_in_progress.pop_completed_batch() {
            self.reserved_fund.record_completed_batch_withdrawal(batch);
        }
    }

    pub(super) fn check_if_withdrawal_completed(&self, batch_id: u64) -> Result<()> {
        self.reserved_fund.check_if_withdrawal_completed(batch_id)
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
