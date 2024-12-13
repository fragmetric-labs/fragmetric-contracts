use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::errors::ErrorCode;

use super::WithdrawalRequest;

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

    /// reserved amount that users can claim for processed withdrawal requests, which is not accounted for as an asset of the fund. for informational purposes only.
    pub sol_user_reserved_amount: u64,

    last_request_id: u64,
    pub last_processed_batch_id: u64,
    last_batch_enqueued_at: i64,
    last_batch_processed_at: i64,

    pending_batch: WithdrawalBatch,

    _padding3: [u8; 15],
    pub num_queued_batches: u8,
    queued_batches: [WithdrawalBatch; MAX_QUEUED_WITHDRAWAL_BATCHES],

    _reserved: [u8; 128],
}

impl WithdrawalState {
    /// 1 fee rate = 1bps = 0.01%
    const WITHDRAWAL_FEE_RATE_BPS_LIMIT: u16 = 500;

    #[inline(always)]
    pub fn get_sol_fee_rate_as_percent(&self) -> f32 {
        self.sol_fee_rate_bps as f32 / 100.0
    }

    #[inline(always)]
    pub fn get_sol_fee_amount(&self, sol_amount: u64) -> Result<u64> {
        crate::utils::get_proportional_amount(sol_amount, self.sol_fee_rate_bps as u64, 10_000)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }

    pub fn set_sol_fee_rate_bps(&mut self, sol_fee_rate_bps: u16) -> Result<()> {
        require_gte!(
            Self::WITHDRAWAL_FEE_RATE_BPS_LIMIT,
            sol_fee_rate_bps,
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

    #[inline(always)]
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

    pub fn create_pending_request(
        &mut self,
        receipt_token_amount: u64,
        current_timestamp: i64,
    ) -> Result<WithdrawalRequest> {
        if self.enabled == 0 {
            err!(ErrorCode::FundWithdrawalDisabledError)?
        }

        // set initial numbers
        if self.last_request_id == 0 {
            self.last_request_id = 10_000;
            self.pending_batch.batch_id = 10_001;
        }

        self.last_request_id += 1;
        let request_id = self.last_request_id;

        let request = WithdrawalRequest::new(
            self.pending_batch.batch_id,
            request_id,
            receipt_token_amount,
            current_timestamp,
        );
        self.pending_batch.add_request(&request)?;

        Ok(request)
    }

    pub fn cancel_pending_request(&mut self, request: WithdrawalRequest) -> Result<()> {
        // assert creation
        require_gte!(
            self.last_request_id,
            request.request_id,
            ErrorCode::FundWithdrawalRequestNotFoundError
        );

        // assert not queued yet
        require_eq!(
            request.batch_id,
            self.pending_batch.batch_id,
            ErrorCode::FundWithdrawalRequestAlreadyQueuedError
        );

        self.pending_batch.remove_request(&request)
    }

    /// returns [enqueued]
    pub fn enqueue_pending_batch(&mut self, current_timestamp: i64, forced: bool) -> bool {
        if !((forced
            || current_timestamp - self.last_batch_enqueued_at
                >= self.batch_threshold_interval_seconds)
            && self.num_queued_batches < MAX_QUEUED_WITHDRAWAL_BATCHES as u8
            && self.pending_batch.num_requests > 0)
        {
            return false;
        }

        let next_batch_id = self.pending_batch.batch_id + 1;
        let mut pending_batch = std::mem::replace(&mut self.pending_batch, {
            let mut new_pending_batch = WithdrawalBatch::zeroed();
            new_pending_batch.initialize(next_batch_id);
            new_pending_batch
        });
        pending_batch.enqueued_at = current_timestamp;

        self.last_batch_enqueued_at = current_timestamp;
        self.queued_batches[self.num_queued_batches as usize] = pending_batch;
        self.num_queued_batches += 1;

        true
    }

    pub fn dequeue_batches(
        &mut self,
        mut count: usize,
        current_timestamp: i64,
    ) -> Result<Vec<WithdrawalBatch>> {
        if count == 0 {
            return Ok(vec![]);
        }
        require_gte!(self.num_queued_batches as usize, count);

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

        Ok(processing_batches)
    }

    pub fn get_queued_batches_iter(&self) -> impl Iterator<Item = &WithdrawalBatch> {
        self.queued_batches[..self.num_queued_batches as usize].iter()
    }

    pub fn get_queued_batches_iter_to_process(
        &self,
        current_timestamp: i64,
        forced: bool,
    ) -> impl Iterator<Item = &WithdrawalBatch> {
        let available = forced
            || current_timestamp - self.last_batch_processed_at
                >= self.batch_threshold_interval_seconds;
        self.get_queued_batches_iter().filter(move |_| available)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod, Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn withdrawal_test() {
        let mut state = WithdrawalState::zeroed();
        assert_eq!(state.pending_batch.batch_id, 0);
        assert_eq!(state.last_request_id, 0);

        assert!(state.create_pending_request(10, 0).is_err());

        state.set_withdrawal_enabled(true);
        state.set_batch_threshold(1).unwrap();

        let req1 = state.create_pending_request(10, 0).unwrap();
        assert_eq!(req1.batch_id, state.pending_batch.batch_id);
        assert_eq!(state.pending_batch.batch_id, 10_001);
        assert_eq!(req1.request_id, 10_001);
        assert_eq!(state.last_request_id, 10_001);
        assert_eq!(state.pending_batch.num_requests, 1);

        let req2 = state.create_pending_request(20, 0).unwrap();
        assert_eq!(req2.batch_id, state.pending_batch.batch_id);
        assert_eq!(req2.request_id, 10_002);
        assert_eq!(state.last_request_id, 10_002);
        assert_eq!(state.pending_batch.num_requests, 2);

        state.cancel_pending_request(req2.clone()).unwrap();
        assert_eq!(state.last_request_id, 10_002);
        assert_eq!(state.pending_batch.num_requests, 1);

        let req3 = state.create_pending_request(20, 0).unwrap();
        assert_eq!(req3.batch_id, state.pending_batch.batch_id);
        assert_eq!(req3.request_id, 10_003);
        assert_eq!(state.last_request_id, 10_003);
        assert_eq!(state.pending_batch.num_requests, 2);

        assert!(!state.enqueue_pending_batch(0, false));
        assert!(state.enqueue_pending_batch(1, false));

        assert!(state.cancel_pending_request(req3).is_err());
        assert_eq!(state.pending_batch.batch_id, 10_002);
        assert_eq!(state.pending_batch.num_requests, 0);
        assert_eq!(state.pending_batch.receipt_token_amount, 0);
        assert_eq!(state.queued_batches[0].batch_id, 10_001);
        assert_eq!(state.queued_batches[0].num_requests, 2);
        assert_eq!(state.queued_batches[0].receipt_token_amount, 30);
        assert_eq!(state.queued_batches[0].enqueued_at, 1);
        assert_eq!(state.last_batch_enqueued_at, 1);
        assert_eq!(state.get_queued_batches_iter().count(), 1);
        assert_eq!(
            state.get_queued_batches_iter_to_process(0, false).count(),
            0
        );
        assert_eq!(state.get_queued_batches_iter_to_process(0, true).count(), 1);
        assert_eq!(
            state.get_queued_batches_iter_to_process(1, false).count(),
            1
        );

        assert!(!state.enqueue_pending_batch(1, false));
        assert!(!state.enqueue_pending_batch(1, true));

        let req4 = state.create_pending_request(30, 2).unwrap();
        assert_eq!(req4.batch_id, state.pending_batch.batch_id);
        assert_eq!(req4.request_id, 10_004);
        assert_eq!(state.last_request_id, 10_004);
        assert_eq!(state.pending_batch.num_requests, 1);

        assert!(state.enqueue_pending_batch(2, false));
        assert_eq!(state.get_queued_batches_iter().count(), 2);
        assert_eq!(state.pending_batch.batch_id, 10_003);
        assert_eq!(state.pending_batch.num_requests, 0);
        assert_eq!(state.pending_batch.receipt_token_amount, 0);
        assert_eq!(state.queued_batches[0].batch_id, 10_001);
        assert_eq!(state.queued_batches[0].num_requests, 2);
        assert_eq!(state.queued_batches[0].receipt_token_amount, 30);
        assert_eq!(state.queued_batches[0].enqueued_at, 1);
        assert_eq!(state.queued_batches[1].batch_id, 10_002);
        assert_eq!(state.queued_batches[1].num_requests, 1);
        assert_eq!(state.queued_batches[1].receipt_token_amount, 30);
        assert_eq!(state.queued_batches[1].enqueued_at, 2);
        assert_eq!(state.last_batch_enqueued_at, 2);
        assert_eq!(state.get_queued_batches_iter().count(), 2);
        assert_eq!(
            state.get_queued_batches_iter_to_process(0, false).count(),
            0
        );
        assert_eq!(state.get_queued_batches_iter_to_process(0, true).count(), 2);
        assert_eq!(
            state.get_queued_batches_iter_to_process(2, false).count(),
            2
        );

        assert!(state.dequeue_batches(3, 3).is_err());
        let dequeued_batches = state.dequeue_batches(2, 3).unwrap();
        assert_eq!(dequeued_batches.len(), 2);
        assert_eq!(dequeued_batches[0].batch_id, 10_001);
        assert_eq!(dequeued_batches[0].num_requests, 2);
        assert_eq!(dequeued_batches[0].receipt_token_amount, 30);
        assert_eq!(dequeued_batches[0].enqueued_at, 1);
        assert_eq!(dequeued_batches[1].batch_id, 10_002);
        assert_eq!(dequeued_batches[1].num_requests, 1);
        assert_eq!(dequeued_batches[1].receipt_token_amount, 30);
        assert_eq!(dequeued_batches[1].enqueued_at, 2);
        assert_eq!(state.last_batch_processed_at, 3);
        assert_eq!(state.last_processed_batch_id, 10_002);

        assert_eq!(state.get_queued_batches_iter().count(), 0);
        assert_eq!(
            state.get_queued_batches_iter_to_process(4, false).count(),
            0
        );
        assert_eq!(state.get_queued_batches_iter_to_process(4, true).count(), 0);

        // println!("{:?}", state);
    }
}
