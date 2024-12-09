use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use std::mem::zeroed;

use crate::errors::ErrorCode;
use crate::modules::fund::SupportedToken;

const MAX_QUEUED_WITHDRAWAL_BATCHES: usize = 10;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod, Debug)]
#[repr(C)]
pub(super) struct WithdrawalState {
    /// configurations
    pub batch_threshold_interval_seconds: i64,
    pub sol_fee_rate_bps: u16,
    pub enabled: u8,
    _padding: [u8; 11],

    /// configuration: basis of normal reserve to cover typical withdrawal volumes rapidly, aiming to minimize redundant circulations and unstaking/unrestaking fees.
    pub sol_normal_reserve_rate_bps: u16,
    pub sol_normal_reserve_max_amount: u64,
    pub sol_withdrawal_reserved_amount: u64,

    /// pending_batch.num_requests + SUM(queued_batches[].num_requests)
    num_requests_in_progress: u64,
    next_batch_id: u64,
    next_request_id: u64,
    pub last_processed_batch_id: u64,
    last_batch_enqueued_at: i64,
    last_batch_processed_at: i64,

    pending_batch: WithdrawalBatch,

    _padding3: [u8; 15],
    num_queued_batches: u8,
    queued_batches: [WithdrawalBatch; MAX_QUEUED_WITHDRAWAL_BATCHES],

    _reserved: [u8; 128],
}

impl WithdrawalState {
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
        self.enabled = if enabled { 1 } else { 0 };
    }

    pub fn set_batch_threshold(&mut self, interval_seconds: i64) -> Result<()> {
        require_gte!(interval_seconds, 0);

        self.batch_threshold_interval_seconds = interval_seconds;

        Ok(())
    }

    pub fn set_sol_normal_reserve_max_amount(&mut self, sol_amount: u64) {
        self.sol_normal_reserve_max_amount = sol_amount;
    }

    pub fn set_sol_normal_reserve_rate_bps(&mut self, reserve_rate_bps: u16) -> Result<()> {
        require_gte!(
            10_00, // 10%
            reserve_rate_bps,
            ErrorCode::FundInvalidUpdateError
        );

        self.sol_normal_reserve_rate_bps = reserve_rate_bps;

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
        if self.enabled == 0 {
            err!(ErrorCode::FundWithdrawalDisabledError)?
        }

        Ok(())
    }

    pub fn is_batch_enqueuing_threshold_satisfied(&self, current_timestamp: i64) -> bool {
        current_timestamp - self.last_batch_enqueued_at > self.batch_threshold_interval_seconds
    }

    pub fn is_batch_processing_threshold_satisfied(&self, current_timestamp: i64) -> bool {
        current_timestamp - self.last_batch_processed_at > self.batch_threshold_interval_seconds
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
        if self.num_queued_batches >= MAX_QUEUED_WITHDRAWAL_BATCHES as u8 {
            return;
        }

        let batch_id = self.next_batch_id;
        self.next_batch_id += 1;
        self.num_queued_batches += 1;
        let new_pending_batch = &mut self.queued_batches[self.num_queued_batches as usize];
        new_pending_batch.initialize(batch_id);

        let mut old_pending_batch = std::mem::replace(&mut self.pending_batch, *new_pending_batch);

        self.num_requests_in_progress += old_pending_batch.num_requests;
        self.last_batch_enqueued_at = current_timestamp;
        old_pending_batch.enqueued_at = current_timestamp;
        self.queued_batches[self.num_queued_batches as usize] = old_pending_batch;
    }

    pub fn dequeue_batches(
        &mut self,
        mut count: usize,
        current_timestamp: i64,
    ) -> Vec<WithdrawalBatch> {
        if count == 0 {
            return vec![];
        }

        count = count.min(self.num_queued_batches as usize);
        self.last_processed_batch_id = self.queued_batches[count - 1].batch_id;
        self.last_batch_processed_at = current_timestamp;
        let processing_batches = self.queued_batches[..count].to_vec();

        for i in 0..self.num_queued_batches as usize {
            if i < (self.num_queued_batches as usize) - count {
                self.queued_batches[i] = self.queued_batches[i + count];
            } else {
                self.queued_batches[i] = WithdrawalBatch::zeroed()
            }
        }
        self.num_queued_batches -= count as u8;

        for processing_batch in &processing_batches {
            self.num_requests_in_progress -= processing_batch.num_requests
        }
        processing_batches
    }

    pub fn get_queued_batches_iter(&self) -> impl Iterator<Item = &WithdrawalBatch> {
        self.queued_batches[..self.num_queued_batches as usize].iter()
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod, Debug, Default)]
#[repr(C)]
pub(super) struct WithdrawalBatch {
    pub batch_id: u64,
    pub num_requests: u64,
    pub receipt_token_amount: u64,
    enqueued_at: i64,
    _reserved: [u8; 32],
}

impl WithdrawalBatch {
    fn initialize(&mut self, batch_id: u64) {
        self.batch_id = batch_id;
        self.num_requests = 0;
        self.receipt_token_amount = 0;
        self.enqueued_at = 0;
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
