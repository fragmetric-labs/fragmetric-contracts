use anchor_lang::prelude::*;

use crate::errors::ErrorCode;

const MAX_QUEUED_WITHDRAWAL_BATCHES: usize = 10;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub(super) struct WithdrawalState {
    pub next_batch_id: u64,
    pub next_request_id: u64,

    /// pending_batch.num_requests + queued_batches[].num_requests
    pub num_requests_in_progress: u64,

    pub last_processed_batch_id: u64,
    pub last_batch_enqueued_at: Option<i64>,
    pub last_batch_processed_at: Option<i64>,

    pub sol_fee_rate_bps: u16,
    pub enabled: bool,
    _padding: [u8; 8],
    pub batch_threshold_interval_seconds: i64,

    pub pending_batch: WithdrawalBatch,
    #[max_len(MAX_QUEUED_WITHDRAWAL_BATCHES)]
    pub queued_batches: Vec<WithdrawalBatch>,

    _reserved: [[u8; 8]; 14],
}

impl WithdrawalState {
    pub fn migrate(&mut self, fund_account_data_version: u16) {
        if fund_account_data_version == 0 {
            self.next_batch_id = 2;
            self.next_request_id = 1;
            self.num_requests_in_progress = 0;
            self.last_processed_batch_id = 0;
            self.last_batch_enqueued_at = None;
            self.last_batch_processed_at = None;
            self.enabled = true;
            self.sol_fee_rate_bps = 0;
            // self.batch_threshold_creation_interval_seconds = 0;
            self._padding = Default::default();
            self.batch_threshold_interval_seconds = 0;
            self.pending_batch = WithdrawalBatch::new(1);
            self.queued_batches = vec![];
            // self.num_completed_withdrawal_requests = 0;
            // self.sol_remaining = 0;
            // self.total_receipt_token_processed = 0;
            // self.total_sol_reserved = 0;
            self._reserved = Default::default();
        } else if fund_account_data_version == 1 {
            // num_completed_withdrawal_requests -> sol_withdrawal_reserved_amount
            // sol_remaining -> sol_fee_income_reserved_amount
            // total_receipt_token_processed: u128 -> receipt_token_processed_amount: u64 & _reserved
            // total_sol_reserved: u128 -> _reserved
            self._reserved = Default::default();
        } else if fund_account_data_version == 2 {
            // sol_fee_income_reserved_amount: u64 -> _padding: [u8; 8]
            self._reserved = Default::default();
        } else if fund_account_data_version == 3 {
            // batch_processing_threshold_amount: u64 -> _padding: [u8; 8]
            self._padding = Default::default();
            self.pending_batch.migrate(fund_account_data_version);
            self.queued_batches
                .iter_mut()
                .for_each(|batch| batch.migrate(fund_account_data_version));
            // sol_withdrawal_reserved_amount: u64 -> _reserved
            // _padding: [u8; 8] -> _reserved
            // receipt_token_processed_amount: u64 -> _reserved
            self._reserved = Default::default();
        }
    }

    /// 1 fee rate = 1bps = 0.01%
    const WITHDRAWAL_FEE_RATE_DIVISOR: u64 = 10_000;
    const WITHDRAWAL_FEE_RATE_LIMIT: u64 = 500;

    #[inline(always)]
    pub fn get_sol_fee_rate_as_percent(&self) -> f32 {
        self.sol_fee_rate_bps as f32 / (Self::WITHDRAWAL_FEE_RATE_DIVISOR / 100) as f32
    }

    #[inline(always)]
    pub fn get_sol_fee_amount(&self, sol_amount: u64) -> Result<u64> {
        crate::utils::get_proportional_amount(
            sol_amount,
            self.sol_fee_rate_bps as u64,
            Self::WITHDRAWAL_FEE_RATE_DIVISOR,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }

    pub fn set_sol_fee_rate_bps(&mut self, sol_fee_rate_bps: u16) -> Result<()> {
        require_gte!(
            Self::WITHDRAWAL_FEE_RATE_LIMIT,
            sol_fee_rate_bps as u64,
            ErrorCode::FundInvalidSolWithdrawalFeeRateError
        );

        self.sol_fee_rate_bps = sol_fee_rate_bps;

        Ok(())
    }

    #[inline(always)]
    pub fn set_withdrawal_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_batch_threshold(&mut self, interval_seconds: i64) -> Result<()> {
        require_gte!(interval_seconds, 0);

        self.batch_threshold_interval_seconds = interval_seconds;

        Ok(())
    }

    pub fn issue_new_withdrawal_request(
        &mut self,
        receipt_token_amount: u64,
        current_timestamp: i64,
    ) -> Result<WithdrawalRequest> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;

        let request = WithdrawalRequest::new(
            self.pending_batch.batch_id,
            request_id,
            receipt_token_amount,
            current_timestamp,
        );
        self.pending_batch.add_request(&request)?;

        Ok(request)
    }

    /// Returns receipt_token_amount
    pub fn remove_withdrawal_request_from_pending_batch(
        &mut self,
        request: WithdrawalRequest,
    ) -> Result<u64> {
        self.assert_request_issued(&request)?;
        self.assert_request_not_enqueued(&request)?;

        self.pending_batch.remove_request(&request)?;

        Ok(request.receipt_token_amount)
    }

    pub fn assert_withdrawal_enabled(&self) -> Result<()> {
        if !self.enabled {
            err!(ErrorCode::FundWithdrawalDisabledError)?
        }

        Ok(())
    }

    pub fn is_batch_enqueuing_threshold_satisfied(&self, current_timestamp: i64) -> bool {
        !self
            .last_batch_enqueued_at
            .is_some_and(|last_batch_enqueued_at| {
                current_timestamp - last_batch_enqueued_at < self.batch_threshold_interval_seconds
            })
    }

    pub fn is_batch_processing_threshold_satisfied(&self, current_timestamp: i64) -> bool {
        !self
            .last_batch_processed_at
            .is_some_and(|last_batch_processed_at| {
                current_timestamp - last_batch_processed_at < self.batch_threshold_interval_seconds
            })
    }

    fn assert_request_issued(&self, request: &WithdrawalRequest) -> Result<()> {
        require_gt!(
            self.next_request_id,
            request.request_id,
            ErrorCode::FundWithdrawalRequestNotFoundError
        );

        Ok(())
    }

    fn assert_request_not_enqueued(&self, request: &WithdrawalRequest) -> Result<()> {
        require_gte!(
            request.batch_id,
            self.pending_batch.batch_id,
            ErrorCode::FundProcessingWithdrawalRequestError
        );

        Ok(())
    }

    pub fn enqueue_pending_batch(&mut self, current_timestamp: i64) {
        if self.queued_batches.len() == MAX_QUEUED_WITHDRAWAL_BATCHES {
            return;
        }

        let batch_id = self.next_batch_id;
        self.next_batch_id += 1;
        let new_pending_batch = WithdrawalBatch::new(batch_id);
        let old_pending_batch = std::mem::replace(&mut self.pending_batch, new_pending_batch);

        self.num_requests_in_progress += old_pending_batch.num_requests;
        self.last_batch_enqueued_at = Some(current_timestamp);
        self.queued_batches.push(old_pending_batch);
    }

    pub fn dequeue_processible_batches(
        &mut self,
        mut available_receipt_token_amount_to_process: u64,
        current_timestamp: i64,
    ) -> Vec<WithdrawalBatch> {
        let mut count = 0;
        for batch in &self.queued_batches {
            if batch.receipt_token_amount <= available_receipt_token_amount_to_process {
                count += 1;
                available_receipt_token_amount_to_process -= batch.receipt_token_amount;
            } else {
                break;
            }
        }

        if count > 0 {
            self.last_processed_batch_id = self.queued_batches[count - 1].batch_id;
            self.last_batch_processed_at = Some(current_timestamp);
        }
        let remaining = self.queued_batches.split_off(count);
        std::mem::replace(&mut self.queued_batches, remaining)
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Default)]
pub(super) struct WithdrawalBatch {
    pub batch_id: u64,
    pub num_requests: u64,
    pub receipt_token_amount: u64,
    _reserved: [[u8; 8]; 8],
    _padding: u8,
}

impl WithdrawalBatch {
    fn migrate(&mut self, fund_account_data_version: u16) {
        if fund_account_data_version == 3 {
            // receipt_token_amount_to_process -> receipt_token_amount
            // receipt_token_being_processed: u64 -> _reserved
            // receipt_token_processed: u64 -> _reserved
            // sol_reserved: u64 -> _reserved
            // procssed_at: Option<i64> -> _reserved + _padding
            self._reserved = Default::default();
            self._padding = Default::default();
        }
    }

    fn new(batch_id: u64) -> Self {
        Self {
            batch_id,
            num_requests: 0,
            receipt_token_amount: 0,
            _reserved: Default::default(),
            _padding: Default::default(),
        }
    }

    fn add_request(&mut self, request: &WithdrawalRequest) -> Result<()> {
        self.num_requests += 1;
        self.receipt_token_amount += request.receipt_token_amount;

        Ok(())
    }

    fn remove_request(&mut self, request: &WithdrawalRequest) -> Result<()> {
        self.num_requests -= 1;
        self.receipt_token_amount -= request.receipt_token_amount;

        Ok(())
    }
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub(super) struct WithdrawalRequest {
    pub batch_id: u64,
    pub request_id: u64,
    pub receipt_token_amount: u64,
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
