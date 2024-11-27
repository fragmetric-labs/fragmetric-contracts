use crate::errors::ErrorCode;
use anchor_lang::prelude::*;
use anchor_spl::token::accessor::amount;

const MAX_QUEUED_BATCH_WITHDRAWALS: usize = 10;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub(super) struct WithdrawalState {
    pub next_batch_id: u64,
    pub next_request_id: u64,

    /// pending_batch.num_requests + queued_batches[].num_requests
    pub num_requests_in_progress: u64,

    pub last_processed_batch_id: u64,
    pub last_batch_created_at: Option<i64>,
    pub last_batch_processed_at: Option<i64>,

    pub sol_fee_rate: u16,
    pub enabled: bool,
    pub batch_threshold_creation_interval_seconds: i64,
    pub batch_threshold_processing_interval_seconds: i64,

    pub pending_batch: WithdrawalBatch,
    #[max_len(MAX_QUEUED_BATCH_WITHDRAWALS)]
    pub queued_batches: Vec<WithdrawalBatch>,

    _reserved: [[u8; 8]; 14],
}

impl WithdrawalState {
    pub(super) fn migrate(&mut self, fund_account_data_version: u16) {
        if fund_account_data_version == 0 {
            self.next_batch_id = 2;
            self.next_request_id = 1;
            self.num_requests_in_progress = 0;
            self.last_processed_batch_id = 0;
            self.last_batch_created_at = None;
            self.last_batch_processed_at = None;
            self.enabled = true;
            self.sol_fee_rate = 0;
            self.batch_threshold_creation_interval_seconds = 0;
            self.batch_threshold_processing_interval_seconds = 0;
            self.pending_batch = WithdrawalBatch::new(1);
            self.queued_batches = vec![];
            self._sol_withdrawal_reserved_amount = Default::default();
            self._padding = Default::default();
            self.processed_receipt_token_amount = Default::default();
            self._reserved = Default::default();
        } else if fund_account_data_version == 1 {
            self._sol_withdrawal_reserved_amount = 0;
            self.processed_receipt_token_amount = 0;
            self._reserved = Default::default();
        } else if fund_account_data_version == 2 {
            self._padding = [0; 8];
        }
    }

    /// 1 fee rate = 1bps = 0.01%
    const WITHDRAWAL_FEE_RATE_DIVISOR: u64 = 10_000;
    const WITHDRAWAL_FEE_RATE_LIMIT: u64 = 500;

    #[inline(always)]
    pub(super) fn get_sol_withdrawal_fee_rate_as_f32(&self) -> f32 {
        self.sol_fee_rate as f32 / (Self::WITHDRAWAL_FEE_RATE_DIVISOR / 100) as f32
    }

    pub(super) fn set_sol_withdrawal_fee_rate(
        &mut self,
        sol_withdrawal_fee_rate: u16,
    ) -> Result<()> {
        require_gte!(
            Self::WITHDRAWAL_FEE_RATE_LIMIT,
            sol_withdrawal_fee_rate as u64,
            ErrorCode::FundInvalidSolWithdrawalFeeRateError
        );

        self.sol_fee_rate = sol_withdrawal_fee_rate;

        Ok(())
    }

    #[inline(always)]
    pub(super) fn set_withdrawal_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub(super) fn set_batch_threshold(
        &mut self,
        creation_interval_seconds: i64,
        processing_interval_seconds: i64,
    ) -> Result<()> {
        require_gte!(creation_interval_seconds, 0);
        require_gte!(processing_interval_seconds, creation_interval_seconds);
        self.batch_threshold_creation_interval_seconds = creation_interval_seconds;
        self.batch_threshold_processing_interval_seconds = processing_interval_seconds;
        Ok(())
    }

    pub(super) fn issue_new_withdrawal_request(
        &mut self,
        receipt_token_amount: u64,
        current_timestamp: i64,
    ) -> Result<WithdrawalRequest> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;

        self.pending_batch
            .add_receipt_token_to_process(receipt_token_amount)?;

        Ok(WithdrawalRequest::new(
            self.pending_batch.batch_id,
            request_id,
            receipt_token_amount,
            current_timestamp,
        ))
    }

    /// Returns receipt_token_amount
    pub(super) fn remove_withdrawal_request_from_batch(
        &mut self,
        request: WithdrawalRequest,
    ) -> Result<u64> {
        self.assert_request_issued(request.request_id)?;

        require_gte!(
            request.batch_id,
            self.pending_batch.batch_id,
            ErrorCode::FundProcessingWithdrawalRequestError
        );

        self.pending_batch
            .remove_receipt_token_to_process(request.receipt_token_amount)?;

        Ok(request.receipt_token_amount)
    }

    /// Returns (sol_user_amount, sol_fee_amount, receipt_token_withdraw_amount)
    pub(super) fn claim_withdrawal_request(
        &mut self,
        request: WithdrawalRequest,
    ) -> Result<(u64, u64, u64)> {
        self.assert_request_issued(request.request_id)?;

        require_gte!(
            self.last_processed_batch_id,
            request.batch_id,
            ErrorCode::FundPendingWithdrawalRequestError
        );

        let sol_amount = crate::utils::get_proportional_amount(
            request.receipt_token_amount,
            self._sol_withdrawal_reserved_amount,
            self.processed_receipt_token_amount,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        let sol_fee_amount = crate::utils::get_proportional_amount(
            sol_amount,
            self.sol_fee_rate as u64,
            Self::WITHDRAWAL_FEE_RATE_DIVISOR,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        let sol_user_amount = sol_amount
            .checked_sub(sol_fee_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        self.processed_receipt_token_amount = self
            .processed_receipt_token_amount
            .checked_sub(request.receipt_token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self._sol_withdrawal_reserved_amount = self
            ._sol_withdrawal_reserved_amount
            .checked_sub(sol_amount)
            .ok_or_else(|| error!(ErrorCode::FundWithdrawalReservedSOLExhaustedException))?;

        Ok((
            sol_user_amount,
            sol_fee_amount,
            request.receipt_token_amount,
        ))
    }

    pub(super) fn assert_withdrawal_enabled(&self) -> Result<()> {
        if !self.enabled {
            err!(ErrorCode::FundWithdrawalDisabledError)?
        }

        Ok(())
    }

    pub(super) fn assert_withdrawal_threshold_satisfied(
        &self,
        current_timestamp: i64,
    ) -> Result<()> {
        let threshold_satisfied = match self.last_batch_created_at {
            Some(last_batch_processing_started_at) => {
                current_timestamp - last_batch_processing_started_at
                    > self.batch_threshold_processing_interval_seconds
            }
            None => true,
        };

        if !threshold_satisfied {
            err!(ErrorCode::OperatorJobUnmetThresholdError)?;
        }

        Ok(())
    }

    fn assert_request_issued(&self, request_id: u64) -> Result<()> {
        require_gt!(
            self.next_request_id,
            request_id,
            ErrorCode::FundWithdrawalRequestNotFoundError
        );

        Ok(())
    }

    pub(super) fn start_processing_pending_batch_withdrawal(
        &mut self,
        current_timestamp: i64,
    ) -> Result<()> {
        require_gt!(
            MAX_QUEUED_BATCH_WITHDRAWALS,
            self.queued_batches.len(),
            ErrorCode::FundExceededMaxBatchWithdrawalInProgressError
        );

        let batch_id = self.next_batch_id;
        self.next_batch_id += 1;
        let new = WithdrawalBatch::new(batch_id);

        let mut old = std::mem::replace(&mut self.pending_batch, new);
        old.processed_at = Some(current_timestamp);

        self.num_requests_in_progress += old.num_requests;
        self.last_batch_created_at = old.processed_at;
        self.queued_batches.push(old);

        Ok(())
    }

    pub(super) fn end_processing_completed_batch_withdrawals(
        &mut self,
        current_timestamp: i64,
    ) -> Result<()> {
        let completed_batch_withdrawals = self.pop_completed_batch_withdrawals_from_queue();
        if let Some(batch) = completed_batch_withdrawals.last() {
            self.last_processed_batch_id = batch.batch_id;
            self.last_batch_processed_at = Some(current_timestamp);
        }

        for batch in completed_batch_withdrawals {
            self.processed_receipt_token_amount = self
                .processed_receipt_token_amount
                .checked_add(batch._receipt_token_processed)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
            self._sol_withdrawal_reserved_amount = self
                ._sol_withdrawal_reserved_amount
                .checked_add(batch.sol_reserved)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        }

        Ok(())
    }

    fn pop_completed_batch_withdrawals_from_queue(&mut self) -> Vec<WithdrawalBatch> {
        let (completed, remaining) = std::mem::take(&mut self.queued_batches)
            .into_iter()
            .partition(|batch| {
                if batch.is_completed() {
                    self.num_requests_in_progress -= batch.num_requests;
                    true
                } else {
                    false
                }
            });
        self.queued_batches = remaining;
        completed
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Default)]
pub struct WithdrawalBatch {
    batch_id: u64,
    num_requests: u64,
    pub(super) receipt_token_amount: u64,
    _receipt_token_being_processed: u64,
    _receipt_token_processed: u64,
    sol_reserved: u64,
    processed_at: Option<i64>,
    _reserved: [u8; 32],
}

impl WithdrawalBatch {
    fn new(batch_id: u64) -> Self {
        Self {
            batch_id,
            num_requests: 0,
            receipt_token_amount: 0,
            _receipt_token_being_processed: 0,
            _receipt_token_processed: 0,
            sol_reserved: 0,
            processed_at: None,
            _reserved: [0; 32],
        }
    }

    fn add_receipt_token_to_process(&mut self, amount: u64) -> Result<()> {
        self.num_requests += 1;
        self.receipt_token_amount = self
            .receipt_token_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }

    fn remove_receipt_token_to_process(&mut self, amount: u64) -> Result<()> {
        self.num_requests -= 1;
        self.receipt_token_amount = self
            .receipt_token_amount
            .checked_sub(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }

    #[inline(always)]
    fn is_completed(&self) -> bool {
        self.processed_at.is_some()
            && self.receipt_token_amount == 0
            && self._receipt_token_being_processed == 0
    }

    pub(super) fn record_unstaking_start(&mut self, receipt_token_amount: u64) -> Result<()> {
        self.receipt_token_amount = self
            .receipt_token_amount
            .checked_sub(receipt_token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self._receipt_token_being_processed = self
            ._receipt_token_being_processed
            .checked_add(receipt_token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }

    pub(super) fn record_unstaking_end(
        &mut self,
        receipt_token_amount: u64,
        sol_amount: u64,
    ) -> Result<()> {
        self._receipt_token_being_processed = self
            ._receipt_token_being_processed
            .checked_sub(receipt_token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self._receipt_token_processed = self
            ._receipt_token_processed
            .checked_add(receipt_token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self.sol_reserved = self
            .sol_reserved
            .checked_add(sol_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct WithdrawalRequest {
    pub(super) batch_id: u64,
    pub(super) request_id: u64,
    receipt_token_amount: u64,
    created_at: i64,
    _reserved: [u8; 16],
}

impl WithdrawalRequest {
    fn new(
        batch_id: u64,
        request_id: u64,
        receipt_token_amount: u64,
        current_timestamp: i64,
    ) -> Self {
        Self {
            batch_id,
            request_id,
            receipt_token_amount,
            created_at: current_timestamp,
            _reserved: [0; 16],
        }
    }
}
