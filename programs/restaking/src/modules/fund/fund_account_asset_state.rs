use anchor_lang::prelude::*;
use bytemuck::Zeroable;

use crate::errors::ErrorCode;
use crate::modules::pricing::{Asset, PricingService, TokenValue};

use super::WithdrawalRequest;

pub const FUND_ACCOUNT_MAX_QUEUED_WITHDRAWAL_BATCHES: usize = 10;

#[zero_copy]
#[repr(C)]
pub(super) struct AssetState {
    token_mint: Pubkey,
    token_program: Pubkey,

    pub accumulated_deposit_capacity_amount: u64,
    pub accumulated_deposit_amount: u64,
    pub depositable: u8,
    _padding: [u8; 4],
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
    _reserved: [u8; 56],

    /// receipt token amount that users can request to withdraw with the given asset from the fund.
    /// it can be conditionally inaccurate on price changes among multiple assets, so make sure to update this properly before any use of it.
    /// do not make any hard limit constraints with this value from off-chain. a requested withdrawal amount will be adjusted on-chain based on the status.
    pub withdrawable_value_as_receipt_token_amount: u64,

    /// informative: reserved amount that users can claim for processed withdrawal requests, which is not accounted for as an asset of the fund.
    pub withdrawal_user_reserved_amount: u64,

    /// asset: receivable amount that the fund may charge the users requesting withdrawals.
    /// It is accrued during either the preparation of the withdrawal obligation or rebalancing of LST like fees from (un)staking or (un)restaking.
    /// And it shall be settled by the withdrawal fee normally. And it also can be written off by a donation operation.
    /// Then it costs the rebalancing expense to the capital of the fund itself as an operation cost instead of charging the users requesting withdrawals.
    pub operation_receivable_amount: u64,

    /// asset: remaining asset for cash-in/out
    pub operation_reserved_amount: u64,
}

impl AssetState {
    pub fn initialize(
        &mut self,
        asset_token_mint_and_program: Option<(Pubkey, Pubkey)>,
        operation_reserved_amount: u64,
    ) {
        *self = Zeroable::zeroed();

        if let Some((token_mint, token_program)) = asset_token_mint_and_program {
            self.token_mint = token_mint;
            self.token_program = token_program;
        }
        self.withdrawal_pending_batch.initialize(1);
        self.operation_reserved_amount = operation_reserved_amount;
    }

    pub fn get_token_mint_and_program(&self) -> Option<(Pubkey, Pubkey)> {
        if self.token_mint == Pubkey::default() {
            None
        } else {
            Some((self.token_mint, self.token_program))
        }
    }

    pub fn set_accumulated_deposit_amount(&mut self, amount: u64) -> Result<&mut Self> {
        require_gte!(
            self.accumulated_deposit_capacity_amount,
            amount,
            ErrorCode::FundInvalidConfigurationUpdateError
        );

        self.accumulated_deposit_amount = amount;

        Ok(self)
    }

    pub fn set_accumulated_deposit_capacity_amount(&mut self, amount: u64) -> Result<&mut Self> {
        require_gte!(
            amount,
            self.accumulated_deposit_amount,
            ErrorCode::FundInvalidConfigurationUpdateError
        );

        self.accumulated_deposit_capacity_amount = amount;

        Ok(self)
    }

    pub fn set_normal_reserve_max_amount(&mut self, amount: u64) -> &mut Self {
        self.normal_reserve_max_amount = amount;
        self
    }

    pub fn set_normal_reserve_rate_bps(&mut self, reserve_rate_bps: u16) -> Result<&mut Self> {
        require_gte!(
            10_00, // 10%
            reserve_rate_bps,
            ErrorCode::FundInvalidConfigurationUpdateError
        );

        self.normal_reserve_rate_bps = reserve_rate_bps;

        Ok(self)
    }

    pub fn set_depositable(&mut self, depositable: bool) -> &mut Self {
        self.depositable = depositable as u8;
        self
    }

    pub fn set_withdrawable(&mut self, withdrawable: bool) -> &mut Self {
        self.withdrawable = withdrawable as u8;
        self
    }

    /// returns [deposited_amount]
    pub fn deposit(&mut self, asset_amount: u64) -> Result<u64> {
        if self.depositable == 0 {
            err!(ErrorCode::FundDepositNotSupportedAsset)?
        }

        self.accumulated_deposit_amount += asset_amount;
        self.operation_reserved_amount += asset_amount;

        require_gte!(
            self.accumulated_deposit_capacity_amount,
            self.accumulated_deposit_amount,
            ErrorCode::FundExceededDepositCapacityAmountError
        );

        Ok(asset_amount)
    }

    /// returns [deposited_amount, offsetted_receivable_amount]
    pub fn donate(&mut self, asset_amount: u64, offset_receivable: bool) -> Result<(u64, u64)> {
        // offset receivable first if requested
        let offsetting_receivable_amount = offset_receivable
            .then(|| self.operation_receivable_amount.min(asset_amount))
            .unwrap_or_default();
        self.operation_receivable_amount -= offsetting_receivable_amount;
        self.operation_reserved_amount += offsetting_receivable_amount;

        let remaining_asset_amount = asset_amount - offsetting_receivable_amount;
        let deposited_amount = if remaining_asset_amount > 0 {
            self.deposit(remaining_asset_amount)?
        } else {
            0
        };

        Ok((deposited_amount, offsetting_receivable_amount))
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
            self.get_token_mint_and_program(),
            current_timestamp,
        );
        self.withdrawal_pending_batch.add_request(&request)?;
        self.withdrawable_value_as_receipt_token_amount = self
            .withdrawable_value_as_receipt_token_amount
            .saturating_sub(receipt_token_amount);

        Ok(request)
    }

    pub fn cancel_withdrawal_request(&mut self, request: &WithdrawalRequest) -> Result<()> {
        // assert not queued yet
        require_eq!(
            request.batch_id,
            self.withdrawal_pending_batch.batch_id,
            ErrorCode::FundWithdrawalRequestAlreadyQueuedError
        );

        self.withdrawal_pending_batch.remove_request(request)?;
        self.withdrawable_value_as_receipt_token_amount = self
            .withdrawable_value_as_receipt_token_amount
            .saturating_add(request.receipt_token_amount);

        Ok(())
    }

    /// returns [enqueued_receipt_token_amount]
    pub fn enqueue_withdrawal_pending_batch(
        &mut self,
        withdrawal_batch_threshold_interval_seconds: i64,
        current_timestamp: i64,
        forced: bool,
    ) -> u64 {
        if !((forced
            || current_timestamp - self.withdrawal_last_batch_enqueued_at
                >= withdrawal_batch_threshold_interval_seconds)
            && self.withdrawal_num_queued_batches
                < FUND_ACCOUNT_MAX_QUEUED_WITHDRAWAL_BATCHES as u8
            && self.withdrawal_pending_batch.num_requests > 0)
        {
            return 0;
        }

        let next_batch_id = self.withdrawal_pending_batch.batch_id + 1;
        let mut pending_batch = std::mem::take(&mut self.withdrawal_pending_batch);
        self.withdrawal_pending_batch.initialize(next_batch_id);
        pending_batch.enqueued_at = current_timestamp;

        self.withdrawal_last_batch_enqueued_at = current_timestamp;
        self.withdrawal_queued_batches[self.withdrawal_num_queued_batches as usize] = pending_batch;
        self.withdrawal_num_queued_batches += 1;

        pending_batch.receipt_token_amount
    }

    pub fn dequeue_withdrawal_batches(
        &mut self,
        count: usize,
        current_timestamp: i64,
    ) -> Result<Vec<WithdrawalBatch>> {
        if count == 0 {
            return Ok(vec![]);
        }
        require_gte!(self.withdrawal_num_queued_batches as usize, count);

        self.withdrawal_last_processed_batch_id =
            self.withdrawal_queued_batches[count - 1].batch_id;
        self.withdrawal_last_batch_processed_at = current_timestamp;
        // take `count` batches from front
        let processing_batches = (0..count)
            .map(|i| std::mem::take(&mut self.withdrawal_queued_batches[i]))
            .collect();
        // then shift to left
        self.withdrawal_queued_batches[..self.withdrawal_num_queued_batches as usize]
            .rotate_left(count);
        self.withdrawal_num_queued_batches -= count as u8;

        Ok(processing_batches)
    }

    fn get_queued_withdrawal_batches_iter(&self) -> impl Iterator<Item = &WithdrawalBatch> {
        self.withdrawal_queued_batches[..self.withdrawal_num_queued_batches as usize].iter()
    }

    pub fn get_queued_withdrawal_batches_to_process_iter(
        &self,
        withdrawal_batch_threshold_interval_seconds: i64,
        current_timestamp: i64,
        forced: bool,
    ) -> impl Iterator<Item = &WithdrawalBatch> {
        let available = forced
            || current_timestamp - self.withdrawal_last_batch_processed_at
                >= withdrawal_batch_threshold_interval_seconds;
        available
            .then(|| self.get_queued_withdrawal_batches_iter())
            .into_iter()
            .flatten()
    }

    /// cash of current asset account
    pub fn get_total_reserved_amount(&self) -> u64 {
        self.operation_reserved_amount + self.withdrawal_user_reserved_amount
    }

    /// total asset amount from given receipt_token_value, so it includes cash, receivable, normalized, restaked assets.
    pub fn get_total_amount(&self, receipt_token_value: &TokenValue) -> u64 {
        let (supported_token_mint, _) = self.get_token_mint_and_program().unzip();
        receipt_token_value
            .numerator
            .iter()
            .find_map(|asset| match (asset, supported_token_mint) {
                (Asset::SOL(sol_amount), None) => Some(*sol_amount),
                (Asset::Token(mint, _, token_amount), Some(supported_token_mint))
                    if supported_token_mint == *mint =>
                {
                    Some(*token_amount)
                }
                _ => None,
            })
            .unwrap_or_default()
    }

    /// receipt token amount in the queued withdrawal batches for an asset.
    pub fn get_receipt_token_withdrawal_obligated_amount(&self) -> u64 {
        self.get_queued_withdrawal_batches_iter()
            .map(|b| b.receipt_token_amount)
            .sum()
    }

    /// receipt token amount in the queued and pending withdrawal batches for an asset.
    pub fn get_receipt_token_withdrawal_requested_amount(&self) -> u64 {
        self.get_receipt_token_withdrawal_obligated_amount()
            + self.withdrawal_pending_batch.receipt_token_amount
    }

    /// based on asset normal reserve configuration, the normal reserve amount relative to total_asset_amount of the fund.
    fn get_withdrawal_normal_reserve_amount(
        &self,
        receipt_token_value: &TokenValue,
    ) -> Result<u64> {
        Ok(crate::utils::get_proportional_amount(
            self.get_total_amount(receipt_token_value),
            self.normal_reserve_rate_bps as u64,
            10_000,
        )?
        .min(self.normal_reserve_max_amount))
    }

    /// get asset amount required for withdrawal in current state, including normal reserve if there is remaining asset_operation_reserved_amount after withdrawal obligation met.
    /// asset_withdrawal_obligated_reserve_amount + MIN(asset_withdrawal_normal_reserve_amount, MAX(0, asset_operation_reserved_amount - asset_withdrawal_obligated_reserve_amount))
    fn get_total_withdrawal_reserve_amount(
        &self,
        receipt_token_mint: &Pubkey,
        receipt_token_value: &TokenValue,
        pricing_service: &PricingService,
        with_normal_reserve: bool,
    ) -> Result<u64> {
        let (supported_token_mint, _) = self.get_token_mint_and_program().unzip();
        let asset_withdrawal_obligated_reserve_amount = pricing_service.get_token_amount_as_asset(
            receipt_token_mint,
            self.get_receipt_token_withdrawal_obligated_amount(),
            supported_token_mint.as_ref(),
        )?;

        Ok(asset_withdrawal_obligated_reserve_amount
            + if with_normal_reserve {
                self.get_withdrawal_normal_reserve_amount(receipt_token_value)?
                    .min(
                        self.operation_reserved_amount
                            .saturating_sub(asset_withdrawal_obligated_reserve_amount),
                    )
            } else {
                0
            })
    }

    /// represents the surplus or shortage amount after fulfilling the withdrawal obligations for the given asset.
    pub fn get_net_operation_reserved_amount(
        &self,
        receipt_token_mint: &Pubkey,
        receipt_token_value: &TokenValue,
        with_normal_reserve: bool,
        pricing_service: &PricingService,
    ) -> Result<i128> {
        Ok(self.operation_reserved_amount as i128
            - self.get_total_withdrawal_reserve_amount(
                receipt_token_mint,
                receipt_token_value,
                pricing_service,
                with_normal_reserve,
            )? as i128)
    }
}

#[zero_copy]
#[repr(C)]
#[derive(Default)]
pub(super) struct WithdrawalBatch {
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

        assert_eq!(
            asset.enqueue_withdrawal_pending_batch(
                withdrawal_batch_threshold_interval_seconds,
                0,
                false
            ),
            0
        );
        assert_ne!(
            asset.enqueue_withdrawal_pending_batch(
                withdrawal_batch_threshold_interval_seconds,
                1,
                false
            ),
            0
        );

        assert!(asset.cancel_withdrawal_request(&req3).is_err());
        assert_eq!(asset.withdrawal_pending_batch.batch_id, 2);
        assert_eq!(asset.withdrawal_pending_batch.num_requests, 0);
        assert_eq!(asset.withdrawal_pending_batch.receipt_token_amount, 0);
        assert_eq!(
            asset
                .get_queued_withdrawal_batches_iter()
                .next()
                .unwrap()
                .batch_id,
            1
        );
        assert_eq!(
            asset
                .get_queued_withdrawal_batches_iter()
                .next()
                .unwrap()
                .num_requests,
            2
        );
        assert_eq!(
            asset
                .get_queued_withdrawal_batches_iter()
                .next()
                .unwrap()
                .receipt_token_amount,
            30
        );
        assert_eq!(
            asset
                .get_queued_withdrawal_batches_iter()
                .next()
                .unwrap()
                .enqueued_at,
            1
        );
        assert_eq!(asset.withdrawal_last_batch_enqueued_at, 1);
        assert_eq!(asset.get_queued_withdrawal_batches_iter().count(), 1);
        assert_eq!(
            asset
                .get_queued_withdrawal_batches_to_process_iter(
                    withdrawal_batch_threshold_interval_seconds,
                    0,
                    false
                )
                .count(),
            0
        );
        assert_eq!(
            asset
                .get_queued_withdrawal_batches_to_process_iter(
                    withdrawal_batch_threshold_interval_seconds,
                    0,
                    true
                )
                .count(),
            1
        );
        assert_eq!(
            asset
                .get_queued_withdrawal_batches_to_process_iter(
                    withdrawal_batch_threshold_interval_seconds,
                    1,
                    false
                )
                .count(),
            1
        );

        assert_eq!(
            asset.enqueue_withdrawal_pending_batch(
                withdrawal_batch_threshold_interval_seconds,
                1,
                false
            ),
            0
        );
        assert_eq!(
            asset.enqueue_withdrawal_pending_batch(
                withdrawal_batch_threshold_interval_seconds,
                1,
                true
            ),
            0
        );

        let req4 = asset.create_withdrawal_request(30, 2).unwrap();
        assert_eq!(req4.batch_id, asset.withdrawal_pending_batch.batch_id);
        assert_eq!(req4.request_id, 4);
        assert_eq!(asset.withdrawal_last_created_request_id, 4);
        assert_eq!(asset.withdrawal_pending_batch.num_requests, 1);

        assert_eq!(
            asset.enqueue_withdrawal_pending_batch(
                withdrawal_batch_threshold_interval_seconds,
                2,
                false
            ),
            30
        );
        assert_eq!(asset.get_queued_withdrawal_batches_iter().count(), 2);
        assert_eq!(asset.withdrawal_pending_batch.batch_id, 3);
        assert_eq!(asset.withdrawal_pending_batch.num_requests, 0);
        assert_eq!(asset.withdrawal_pending_batch.receipt_token_amount, 0);
        assert_eq!(
            asset
                .get_queued_withdrawal_batches_iter()
                .next()
                .unwrap()
                .batch_id,
            1
        );
        assert_eq!(
            asset
                .get_queued_withdrawal_batches_iter()
                .next()
                .unwrap()
                .num_requests,
            2
        );
        assert_eq!(
            asset
                .get_queued_withdrawal_batches_iter()
                .next()
                .unwrap()
                .receipt_token_amount,
            30
        );
        assert_eq!(
            asset
                .get_queued_withdrawal_batches_iter()
                .next()
                .unwrap()
                .enqueued_at,
            1
        );
        assert_eq!(
            asset
                .get_queued_withdrawal_batches_iter()
                .skip(1)
                .next()
                .unwrap()
                .batch_id,
            2
        );
        assert_eq!(
            asset
                .get_queued_withdrawal_batches_iter()
                .skip(1)
                .next()
                .unwrap()
                .num_requests,
            1
        );
        assert_eq!(
            asset
                .get_queued_withdrawal_batches_iter()
                .skip(1)
                .next()
                .unwrap()
                .receipt_token_amount,
            30
        );
        assert_eq!(
            asset
                .get_queued_withdrawal_batches_iter()
                .skip(1)
                .next()
                .unwrap()
                .enqueued_at,
            2
        );
        assert_eq!(asset.withdrawal_last_batch_enqueued_at, 2);
        assert_eq!(asset.get_queued_withdrawal_batches_iter().count(), 2);
        assert_eq!(
            asset
                .get_queued_withdrawal_batches_to_process_iter(
                    withdrawal_batch_threshold_interval_seconds,
                    0,
                    false
                )
                .count(),
            0
        );
        assert_eq!(
            asset
                .get_queued_withdrawal_batches_to_process_iter(
                    withdrawal_batch_threshold_interval_seconds,
                    0,
                    true
                )
                .count(),
            2
        );
        assert_eq!(
            asset
                .get_queued_withdrawal_batches_to_process_iter(
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

        assert_eq!(asset.get_queued_withdrawal_batches_iter().count(), 0);
        assert_eq!(
            asset
                .get_queued_withdrawal_batches_to_process_iter(
                    withdrawal_batch_threshold_interval_seconds,
                    4,
                    false
                )
                .count(),
            0
        );
        assert_eq!(
            asset
                .get_queued_withdrawal_batches_to_process_iter(
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
