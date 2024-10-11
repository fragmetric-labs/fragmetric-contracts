use anchor_lang::prelude::*;
use crate::errors::ErrorCode;
use crate::modules::fund::{
    BatchWithdrawal, ReservedFund, UserFundAccount, WithdrawalRequest, WithdrawalStatus,
};

impl BatchWithdrawal {
    fn add_receipt_token_to_process(&mut self, amount: u64) -> Result<()> {
        self.num_withdrawal_requests += 1;
        self.receipt_token_to_process = self
            .receipt_token_to_process
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }

    fn remove_receipt_token_to_process(&mut self, amount: u64) -> Result<()> {
        self.num_withdrawal_requests -= 1;
        self.receipt_token_to_process = self
            .receipt_token_to_process
            .checked_sub(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }

    fn start_batch_processing(&mut self) -> Result<()> {
        self.processing_started_at = Some(crate::utils::timestamp_now()?);
        Ok(())
    }

    fn is_completed(&self) -> bool {
        self.processing_started_at.is_some()
            && self.receipt_token_to_process == 0
            && self.receipt_token_being_processed == 0
    }

    // Called by operator
    pub fn record_unstaking_start(&mut self, receipt_token_amount: u64) -> Result<()> {
        self.receipt_token_to_process = self
            .receipt_token_to_process
            .checked_sub(receipt_token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self.receipt_token_being_processed = self
            .receipt_token_being_processed
            .checked_add(receipt_token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }

    // Called by operator
    pub fn record_unstaking_end(
        &mut self,
        receipt_token_amount: u64,
        sol_amount: u64,
    ) -> Result<()> {
        self.receipt_token_being_processed = self
            .receipt_token_being_processed
            .checked_sub(receipt_token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self.receipt_token_processed = self
            .receipt_token_processed
            .checked_add(receipt_token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self.sol_reserved = self
            .sol_reserved
            .checked_add(sol_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }
}

impl ReservedFund {
    fn record_completed_batch_withdrawal(&mut self, batch: BatchWithdrawal) -> Result<()> {
        self.receipt_token_processed_amount = self
            .receipt_token_processed_amount
            .checked_add(batch.receipt_token_processed)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self.sol_withdrawal_reserved_amount = self
            .sol_withdrawal_reserved_amount
            .checked_add(batch.sol_reserved)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }

    pub fn calculate_sol_amount_for_receipt_token_amount(
        &self,
        receipt_token_withdraw_amount: u64,
    ) -> Result<u64> {
        crate::utils::proportional_amount(
            receipt_token_withdraw_amount,
            self.sol_withdrawal_reserved_amount,
            self.receipt_token_processed_amount,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }

    fn withdraw(
        &mut self,
        sol_amount: u64,
        sol_fee_amount: u64,
        burned_receipt_token_amount: u64,
    ) -> Result<()> {
        self.receipt_token_processed_amount = self
            .receipt_token_processed_amount
            .checked_sub(burned_receipt_token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self.sol_withdrawal_reserved_amount = self
            .sol_withdrawal_reserved_amount
            .checked_sub(sol_amount)
            .ok_or_else(|| error!(ErrorCode::FundWithdrawalReservedSOLExhaustedException))?;

        // send fee to fee income
        self.sol_fee_income_reserved_amount = self
            .sol_fee_income_reserved_amount
            .checked_add(sol_fee_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }
}

impl WithdrawalStatus {
    pub fn create_withdrawal_request(
        &mut self,
        receipt_token_amount: u64,
    ) -> Result<WithdrawalRequest> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;

        self.pending_batch_withdrawal
            .add_receipt_token_to_process(receipt_token_amount)?;
        WithdrawalRequest::new(
            self.pending_batch_withdrawal.batch_id,
            request_id,
            receipt_token_amount,
        )
    }

    pub fn remove_withdrawal_request(&mut self, receipt_token_amount: u64) -> Result<()> {
        self.pending_batch_withdrawal
            .remove_receipt_token_to_process(receipt_token_amount)
    }

    pub fn calculate_sol_withdrawal_fee(&self, amount: u64) -> Result<u64> {
        crate::utils::proportional_amount(
            amount,
            self.sol_withdrawal_fee_rate as u64,
            Self::WITHDRAWAL_FEE_RATE_DIVISOR,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }

    pub fn withdraw(
        &mut self,
        sol_amount: u64,
        sol_fee_amount: u64,
        burned_receipt_token_amount: u64,
    ) -> Result<()> {
        self.reserved_fund
            .withdraw(sol_amount, sol_fee_amount, burned_receipt_token_amount)
    }

    pub fn check_withdrawal_enabled(&self) -> Result<()> {
        if !self.withdrawal_enabled_flag {
            err!(ErrorCode::FundWithdrawalDisabledError)?
        }

        Ok(())
    }

    pub fn check_batch_processing_not_started(&self, batch_id: u64) -> Result<()> {
        if batch_id < self.pending_batch_withdrawal.batch_id {
            err!(ErrorCode::FundProcessingWithdrawalRequestError)?
        }

        Ok(())
    }

    pub fn check_batch_processing_completed(&self, batch_id: u64) -> Result<()> {
        if batch_id > self.last_completed_batch_id {
            err!(ErrorCode::FundPendingWithdrawalRequestError)?
        }

        Ok(())
    }

    // Called by operator
    pub fn start_processing_pending_batch_withdrawal(&mut self) -> Result<()> {
        let batch_id = self.next_batch_id;
        self.next_batch_id += 1;
        let new = BatchWithdrawal::new(batch_id);

        let mut old = std::mem::replace(&mut self.pending_batch_withdrawal, new);
        old.start_batch_processing()?;

        self.num_withdrawal_requests_in_progress += old.num_withdrawal_requests;
        self.last_batch_processing_started_at = old.processing_started_at;
        self.batch_withdrawals_in_progress.push(old);

        Ok(())
    }

    // Called by operator
    pub fn end_processing_completed_batch_withdrawals(&mut self) -> Result<()> {
        let completed_batch_withdrawals = self.pop_completed_batch_withdrawals();
        if let Some(batch) = completed_batch_withdrawals.last() {
            self.last_completed_batch_id = batch.batch_id;
            self.last_batch_processing_completed_at = Some(crate::utils::timestamp_now()?);
        }
        for batch in completed_batch_withdrawals {
            self.reserved_fund
                .record_completed_batch_withdrawal(batch)?;
        }

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

impl UserFundAccount {
    pub fn push_withdrawal_request(&mut self, request: WithdrawalRequest) -> Result<()> {
        if self.withdrawal_requests.len() == Self::MAX_WITHDRAWAL_REQUESTS_SIZE {
            err!(ErrorCode::FundExceededMaxWithdrawalRequestError)?;
        }

        self.withdrawal_requests.push(request);

        Ok(())
    }

    pub fn pop_withdrawal_request(&mut self, request_id: u64) -> Result<WithdrawalRequest> {
        let index = self
            .withdrawal_requests
            .binary_search_by_key(&request_id, |req| req.request_id)
            .map_err(|_| error!(ErrorCode::FundWithdrawalRequestNotFoundError))?;
        Ok(self.withdrawal_requests.remove(index))
    }
}
