use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use super::WithdrawalRequest;
use crate::errors::ErrorCode;
use crate::modules::pricing::PricingService;
use crate::utils::get_proportional_amount;

pub const FUND_ACCOUNT_MAX_QUEUED_WITHDRAWAL_BATCHES: usize = 10;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod, Debug)]
#[repr(C)]
pub struct AssetState {
    token_mint: Pubkey, // Pubkey::default() for SOL
    pub accumulated_deposit_capacity_amount: u64,
    pub accumulated_deposit_amount: u64,
    _padding: [u8; 5],
    pub withdrawable: u8,
    pub normal_reserve_rate_bps: u16,
    pub normal_reserve_max_amount: u64,

    pub withdrawal_last_created_request_id: u64,
    pub withdrawal_last_processed_batch_id: u64,
    pub withdrawal_last_batch_enqueued_at: i64,
    pub withdrawal_last_batch_processed_at: i64,

    pub withdrawal_pending_batch: WithdrawalBatch,
    _padding2: [u8; 15],
    withdrawal_num_queued_batches: u8,
    withdrawal_queued_batches: [WithdrawalBatch; FUND_ACCOUNT_MAX_QUEUED_WITHDRAWAL_BATCHES],
    _reserved: [u8; 64],

    /// informative: reserved amount that users can claim for processed withdrawal requests, which is not accounted for as an asset of the fund.
    pub withdrawal_user_reserved_amount: u64,

    /// asset: A receivable that the fund may charge the users requesting withdrawals.
    /// It is accrued during either the preparation of the withdrawal obligation or rebalancing of LST (fee from unstaking, unrestaking).
    /// And it shall be settled by the withdrawal fee normally. But it also can be written off by an authorized operation.
    /// Then it costs the rebalancing expense to the capital of the fund itself as an operation cost instead of charging the users requesting withdrawals.
    pub operation_receivable_amount: u64,

    /// asset
    pub operation_reserved_amount: u64,
}

impl AssetState {
    pub fn initialize(&mut self, asset_token_mint: Option<Pubkey>, operation_reserved_amount: u64) {
        self.token_mint = asset_token_mint.unwrap_or_default();
        self.withdrawal_pending_batch.initialize(1);
        self.operation_reserved_amount = operation_reserved_amount;
    }

    pub fn get_token_mint(&self) -> Option<Pubkey> {
        if self.token_mint == Pubkey::default() {
            None
        } else {
            Some(self.token_mint)
        }
    }

    pub fn set_accumulated_deposit_amount(&mut self, amount: u64) -> Result<()> {
        require_gte!(
            self.accumulated_deposit_capacity_amount,
            amount,
            ErrorCode::FundInvalidUpdateError
        );

        self.accumulated_deposit_amount = amount;

        Ok(())
    }

    pub fn set_accumulated_deposit_capacity_amount(&mut self, amount: u64) -> Result<()> {
        require_gte!(
            amount,
            self.accumulated_deposit_amount,
            ErrorCode::FundInvalidUpdateError
        );

        self.accumulated_deposit_capacity_amount = amount;

        Ok(())
    }

    #[inline(always)]
    pub fn set_normal_reserve_max_amount(&mut self, amount: u64) {
        self.normal_reserve_max_amount = amount;
    }

    pub fn set_normal_reserve_rate_bps(&mut self, reserve_rate_bps: u16) -> Result<()> {
        require_gte!(
            10_00, // 10%
            reserve_rate_bps,
            ErrorCode::FundInvalidUpdateError
        );

        self.normal_reserve_rate_bps = reserve_rate_bps;

        Ok(())
    }

    #[inline(always)]
    pub fn set_withdrawable(&mut self, withdrawable: bool) {
        self.withdrawable = if withdrawable { 1 } else { 0 };
    }

    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let new_accumulated_deposit_amount = self.accumulated_deposit_amount + amount;
        require_gte!(
            self.accumulated_deposit_capacity_amount,
            new_accumulated_deposit_amount,
            ErrorCode::FundExceededDepositCapacityAmountError
        );

        self.accumulated_deposit_amount = new_accumulated_deposit_amount;
        self.operation_reserved_amount = self.operation_reserved_amount + amount;

        Ok(())
    }

    pub fn create_withdrawal_request(
        &mut self,
        receipt_token_amount: u64,
        current_timestamp: i64,
    ) -> Result<WithdrawalRequest> {
        if self.withdrawable == 0 {
            err!(ErrorCode::FundWithdrawalNotSupportedAsset)?
        }

        self.withdrawal_last_created_request_id += 1;
        let request = WithdrawalRequest::new(
            self.withdrawal_pending_batch.batch_id,
            self.withdrawal_last_created_request_id,
            receipt_token_amount,
            if self.token_mint == Pubkey::default() {
                None
            } else {
                Some(self.token_mint)
            },
            current_timestamp,
        );
        self.withdrawal_pending_batch.add_request(&request)?;

        Ok(request)
    }

    pub fn cancel_withdrawal_request(&mut self, request: &WithdrawalRequest) -> Result<()> {
        // assert not queued yet
        require_eq!(
            request.batch_id,
            self.withdrawal_pending_batch.batch_id,
            ErrorCode::FundWithdrawalRequestAlreadyQueuedError
        );

        self.withdrawal_pending_batch.remove_request(request)
    }

    /// returns [enqueued]
    pub fn enqueue_withdrawal_pending_batch(
        &mut self,
        withdrawal_batch_threshold_interval_seconds: i64,
        current_timestamp: i64,
        forced: bool,
    ) -> bool {
        if !((forced
            || current_timestamp - self.withdrawal_last_batch_enqueued_at
                >= withdrawal_batch_threshold_interval_seconds)
            && self.withdrawal_num_queued_batches
                < FUND_ACCOUNT_MAX_QUEUED_WITHDRAWAL_BATCHES as u8
            && self.withdrawal_pending_batch.num_requests > 0)
        {
            return false;
        }

        let next_batch_id = self.withdrawal_pending_batch.batch_id + 1;
        let mut pending_batch = std::mem::replace(&mut self.withdrawal_pending_batch, {
            let mut new_pending_batch = WithdrawalBatch::zeroed();
            new_pending_batch.initialize(next_batch_id);
            new_pending_batch
        });
        pending_batch.enqueued_at = current_timestamp;

        self.withdrawal_last_batch_enqueued_at = current_timestamp;
        self.withdrawal_queued_batches[self.withdrawal_num_queued_batches as usize] = pending_batch;
        self.withdrawal_num_queued_batches += 1;

        true
    }

    pub fn dequeue_withdrawal_batches(
        &mut self,
        mut count: usize,
        current_timestamp: i64,
    ) -> Result<Vec<WithdrawalBatch>> {
        if count == 0 {
            return Ok(vec![]);
        }
        require_gte!(self.withdrawal_num_queued_batches as usize, count);

        self.withdrawal_last_processed_batch_id =
            self.withdrawal_queued_batches[count - 1].batch_id;
        self.withdrawal_last_batch_processed_at = current_timestamp;
        let processing_batches = self.withdrawal_queued_batches[..count].to_vec();

        for i in 0..self.withdrawal_num_queued_batches as usize {
            if i < (self.withdrawal_num_queued_batches as usize) - count {
                self.withdrawal_queued_batches[i] = self.withdrawal_queued_batches[i + count];
            } else {
                self.withdrawal_queued_batches[i] = WithdrawalBatch::zeroed();
            }
        }
        self.withdrawal_num_queued_batches -= count as u8;

        Ok(processing_batches)
    }

    pub fn get_withdrawal_queued_batches_iter(&self) -> impl Iterator<Item = &WithdrawalBatch> {
        self.withdrawal_queued_batches[..self.withdrawal_num_queued_batches as usize].iter()
    }

    pub fn get_withdrawal_queued_batches_iter_to_process(
        &self,
        withdrawal_batch_threshold_interval_seconds: i64,
        current_timestamp: i64,
        forced: bool,
    ) -> impl Iterator<Item = &WithdrawalBatch> {
        let available = forced
            || current_timestamp - self.withdrawal_last_batch_processed_at
                >= withdrawal_batch_threshold_interval_seconds;
        self.get_withdrawal_queued_batches_iter()
            .filter(move |_| available)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod, Debug)]
#[repr(C)]
pub struct WithdrawalBatch {
    pub batch_id: u64,
    pub num_requests: u64,
    pub receipt_token_amount: u64,
    pub enqueued_at: i64,
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
    fn withdraw_basic_test() {
        let mut asset = AssetState::zeroed();
        asset.initialize(None, 0);
        assert_eq!(asset.token_mint, Pubkey::default());
        assert_eq!(asset.withdrawal_pending_batch.batch_id, 1);
        assert_eq!(asset.withdrawal_last_created_request_id, 0);
        assert!(asset.create_withdrawal_request(10, 0).is_err());

        asset.set_withdrawable(true);
        let withdrawal_batch_threshold_interval_seconds = 1;

        let req1 = asset.create_withdrawal_request(10, 0).unwrap();
        assert_eq!(req1.batch_id, asset.withdrawal_pending_batch.batch_id);
        assert_eq!(asset.withdrawal_pending_batch.batch_id, 1);
        assert_eq!(req1.request_id, 1);
        assert_eq!(asset.withdrawal_last_created_request_id, 1);
        assert_eq!(asset.withdrawal_pending_batch.num_requests, 1);

        let req2 = asset.create_withdrawal_request(20, 0).unwrap();
        assert_eq!(req2.batch_id, asset.withdrawal_pending_batch.batch_id);
        assert_eq!(req2.request_id, 2);
        assert_eq!(asset.withdrawal_last_created_request_id, 2);
        assert_eq!(asset.withdrawal_pending_batch.num_requests, 2);

        asset.cancel_withdrawal_request(&req2).unwrap();
        assert_eq!(asset.withdrawal_last_created_request_id, 2);
        assert_eq!(asset.withdrawal_pending_batch.num_requests, 1);

        let req3 = asset.create_withdrawal_request(20, 0).unwrap();
        assert_eq!(req3.batch_id, asset.withdrawal_pending_batch.batch_id);
        assert_eq!(req3.request_id, 3);
        assert_eq!(asset.withdrawal_last_created_request_id, 3);
        assert_eq!(asset.withdrawal_pending_batch.num_requests, 2);

        assert!(!asset.enqueue_withdrawal_pending_batch(
            withdrawal_batch_threshold_interval_seconds,
            0,
            false
        ));
        assert!(asset.enqueue_withdrawal_pending_batch(
            withdrawal_batch_threshold_interval_seconds,
            1,
            false
        ));

        assert!(asset.cancel_withdrawal_request(&req3).is_err());
        assert_eq!(asset.withdrawal_pending_batch.batch_id, 2);
        assert_eq!(asset.withdrawal_pending_batch.num_requests, 0);
        assert_eq!(asset.withdrawal_pending_batch.receipt_token_amount, 0);
        assert_eq!(
            asset
                .get_withdrawal_queued_batches_iter()
                .next()
                .unwrap()
                .batch_id,
            1
        );
        assert_eq!(
            asset
                .get_withdrawal_queued_batches_iter()
                .next()
                .unwrap()
                .num_requests,
            2
        );
        assert_eq!(
            asset
                .get_withdrawal_queued_batches_iter()
                .next()
                .unwrap()
                .receipt_token_amount,
            30
        );
        assert_eq!(
            asset
                .get_withdrawal_queued_batches_iter()
                .next()
                .unwrap()
                .enqueued_at,
            1
        );
        assert_eq!(asset.withdrawal_last_batch_enqueued_at, 1);
        assert_eq!(asset.get_withdrawal_queued_batches_iter().count(), 1);
        assert_eq!(
            asset
                .get_withdrawal_queued_batches_iter_to_process(
                    withdrawal_batch_threshold_interval_seconds,
                    0,
                    false
                )
                .count(),
            0
        );
        assert_eq!(
            asset
                .get_withdrawal_queued_batches_iter_to_process(
                    withdrawal_batch_threshold_interval_seconds,
                    0,
                    true
                )
                .count(),
            1
        );
        assert_eq!(
            asset
                .get_withdrawal_queued_batches_iter_to_process(
                    withdrawal_batch_threshold_interval_seconds,
                    1,
                    false
                )
                .count(),
            1
        );

        assert!(!asset.enqueue_withdrawal_pending_batch(
            withdrawal_batch_threshold_interval_seconds,
            1,
            false
        ));
        assert!(!asset.enqueue_withdrawal_pending_batch(
            withdrawal_batch_threshold_interval_seconds,
            1,
            true
        ));

        let req4 = asset.create_withdrawal_request(30, 2).unwrap();
        assert_eq!(req4.batch_id, asset.withdrawal_pending_batch.batch_id);
        assert_eq!(req4.request_id, 4);
        assert_eq!(asset.withdrawal_last_created_request_id, 4);
        assert_eq!(asset.withdrawal_pending_batch.num_requests, 1);

        assert!(asset.enqueue_withdrawal_pending_batch(
            withdrawal_batch_threshold_interval_seconds,
            2,
            false
        ));
        assert_eq!(asset.get_withdrawal_queued_batches_iter().count(), 2);
        assert_eq!(asset.withdrawal_pending_batch.batch_id, 3);
        assert_eq!(asset.withdrawal_pending_batch.num_requests, 0);
        assert_eq!(asset.withdrawal_pending_batch.receipt_token_amount, 0);
        assert_eq!(
            asset
                .get_withdrawal_queued_batches_iter()
                .next()
                .unwrap()
                .batch_id,
            1
        );
        assert_eq!(
            asset
                .get_withdrawal_queued_batches_iter()
                .next()
                .unwrap()
                .num_requests,
            2
        );
        assert_eq!(
            asset
                .get_withdrawal_queued_batches_iter()
                .next()
                .unwrap()
                .receipt_token_amount,
            30
        );
        assert_eq!(
            asset
                .get_withdrawal_queued_batches_iter()
                .next()
                .unwrap()
                .enqueued_at,
            1
        );
        assert_eq!(
            asset
                .get_withdrawal_queued_batches_iter()
                .skip(1)
                .next()
                .unwrap()
                .batch_id,
            2
        );
        assert_eq!(
            asset
                .get_withdrawal_queued_batches_iter()
                .skip(1)
                .next()
                .unwrap()
                .num_requests,
            1
        );
        assert_eq!(
            asset
                .get_withdrawal_queued_batches_iter()
                .skip(1)
                .next()
                .unwrap()
                .receipt_token_amount,
            30
        );
        assert_eq!(
            asset
                .get_withdrawal_queued_batches_iter()
                .skip(1)
                .next()
                .unwrap()
                .enqueued_at,
            2
        );
        assert_eq!(asset.withdrawal_last_batch_enqueued_at, 2);
        assert_eq!(asset.get_withdrawal_queued_batches_iter().count(), 2);
        assert_eq!(
            asset
                .get_withdrawal_queued_batches_iter_to_process(
                    withdrawal_batch_threshold_interval_seconds,
                    0,
                    false
                )
                .count(),
            0
        );
        assert_eq!(
            asset
                .get_withdrawal_queued_batches_iter_to_process(
                    withdrawal_batch_threshold_interval_seconds,
                    0,
                    true
                )
                .count(),
            2
        );
        assert_eq!(
            asset
                .get_withdrawal_queued_batches_iter_to_process(
                    withdrawal_batch_threshold_interval_seconds,
                    2,
                    false
                )
                .count(),
            2
        );

        assert!(asset.dequeue_withdrawal_batches(3, 3).is_err());
        let dequeued_batches = asset.dequeue_withdrawal_batches(2, 3).unwrap();
        assert_eq!(dequeued_batches.len(), 2);
        assert_eq!(dequeued_batches[0].batch_id, 1);
        assert_eq!(dequeued_batches[0].num_requests, 2);
        assert_eq!(dequeued_batches[0].receipt_token_amount, 30);
        assert_eq!(dequeued_batches[0].enqueued_at, 1);
        assert_eq!(dequeued_batches[1].batch_id, 2);
        assert_eq!(dequeued_batches[1].num_requests, 1);
        assert_eq!(dequeued_batches[1].receipt_token_amount, 30);
        assert_eq!(dequeued_batches[1].enqueued_at, 2);
        assert_eq!(asset.withdrawal_last_batch_processed_at, 3);
        assert_eq!(asset.withdrawal_last_processed_batch_id, 2);

        assert_eq!(asset.get_withdrawal_queued_batches_iter().count(), 0);
        assert_eq!(
            asset
                .get_withdrawal_queued_batches_iter_to_process(
                    withdrawal_batch_threshold_interval_seconds,
                    4,
                    false
                )
                .count(),
            0
        );
        assert_eq!(
            asset
                .get_withdrawal_queued_batches_iter_to_process(
                    withdrawal_batch_threshold_interval_seconds,
                    4,
                    true
                )
                .count(),
            0
        );

        // println!("{:?}", state);
    }
}
