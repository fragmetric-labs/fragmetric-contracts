use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::pricing::{self, TokenPricingSource, TokenPricingSourceMap};
use crate::utils::PDASeeds;

#[constant]
/// ## Version History
/// * v1: Initial Version
/// * v2: Change reserve fund structure
pub const FUND_ACCOUNT_CURRENT_VERSION: u16 = 2;

const MAX_SUPPORTED_TOKENS: usize = 16;

#[account]
#[derive(InitSpace)]
pub struct FundAccount {
    data_version: u16,
    bump: u8,
    pub receipt_token_mint: Pubkey,
    #[max_len(MAX_SUPPORTED_TOKENS)]
    supported_tokens: Vec<SupportedTokenInfo>,
    sol_capacity_amount: u64,
    sol_accumulated_deposit_amount: u64,
    // TODO visibility is currently set to `in crate::modules` due to operator module - change to private and use getter
    pub(in crate::modules) sol_operation_reserved_amount: u64,
    // TODO visibility is currently set to `in crate::modules` due to operator module - change to `super`
    pub(in crate::modules) withdrawal: WithdrawalStatus,
    _reserved: [u8; 1280],
}

impl PDASeeds<2> for FundAccount {
    const SEED: &'static [u8] = b"fund";

    fn get_seeds(&self) -> [&[u8]; 2] {
        [Self::SEED, self.receipt_token_mint.as_ref()]
    }

    fn get_bump_ref(&self) -> &u8 {
        &self.bump
    }
}

impl FundAccount {
    pub const EXECUTION_RESERVED_SEED: &'static [u8] = b"fund_execution_reserved";

    pub(super) fn initialize(&mut self, bump: u8, receipt_token_mint: Pubkey) {
        if self.data_version == 0 {
            self.data_version = 1;
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
            self.withdrawal.initialize();
        }

        if self.data_version == 1 {
            self.data_version = 2;
            self.withdrawal.sol_withdrawal_reserved_amount = 0;
            self.withdrawal.receipt_token_processed_amount = 0;
            self.withdrawal._reserved = [0; 88];
        }
    }

    #[inline(always)]
    pub(super) fn update_if_needed(&mut self, receipt_token_mint: Pubkey) {
        self.initialize(self.bump, receipt_token_mint);
    }

    #[inline(always)]
    pub fn is_latest_version(&self) -> bool {
        self.data_version == FUND_ACCOUNT_CURRENT_VERSION
    }

    #[inline(always)]
    pub(in crate::modules) fn get_supported_tokens_iter(
        &self,
    ) -> impl Iterator<Item = &SupportedTokenInfo> {
        self.supported_tokens.iter()
    }

    #[inline(always)]
    pub(super) fn get_supported_tokens_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut SupportedTokenInfo> {
        self.supported_tokens.iter_mut()
    }

    pub(super) fn get_supported_token(&self, token: Pubkey) -> Result<&SupportedTokenInfo> {
        self.supported_tokens
            .iter()
            .find(|info| info.mint == token)
            .ok_or_else(|| error!(ErrorCode::FundNotSupportedTokenError))
    }

    pub(in crate::modules) fn get_supported_token_mut(
        &mut self,
        token: Pubkey,
    ) -> Result<&mut SupportedTokenInfo> {
        self.supported_tokens
            .iter_mut()
            .find(|info| info.mint == token)
            .ok_or_else(|| error!(ErrorCode::FundNotSupportedTokenError))
    }

    #[inline(always)]
    pub(super) fn get_sol_capacity_amount(&self) -> u64 {
        self.sol_capacity_amount
    }

    #[inline(always)]
    pub(super) fn get_sol_accumulated_deposit_amount(&self) -> u64 {
        self.sol_accumulated_deposit_amount
    }

    #[inline(always)]
    pub(in crate::modules) fn get_sol_operation_reserved_amount(&self) -> u64 {
        self.sol_operation_reserved_amount
    }

    pub(super) fn set_sol_capacity_amount(&mut self, capacity_amount: u64) -> Result<()> {
        require_gte!(
            capacity_amount,
            self.sol_accumulated_deposit_amount,
            ErrorCode::FundInvalidUpdateError
        );

        self.sol_capacity_amount = capacity_amount;

        Ok(())
    }

    pub(super) fn add_supported_token(
        &mut self,
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        capacity_amount: u64,
        pricing_source: TokenPricingSource,
    ) -> Result<()> {
        if self.supported_tokens.iter().any(|info| info.mint == mint) {
            err!(ErrorCode::FundAlreadySupportedTokenError)?
        }

        require_gt!(
            MAX_SUPPORTED_TOKENS,
            self.supported_tokens.len(),
            ErrorCode::FundExceededMaxSupportedTokensError
        );

        let token_info =
            SupportedTokenInfo::new(mint, program, decimals, capacity_amount, pricing_source);
        self.supported_tokens.push(token_info);

        Ok(())
    }

    pub(super) fn deposit_sol(&mut self, amount: u64) -> Result<()> {
        let new_sol_accumulated_deposit_amount = self
            .sol_accumulated_deposit_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        require_gte!(
            self.sol_capacity_amount,
            new_sol_accumulated_deposit_amount,
            ErrorCode::FundExceededSOLCapacityAmountError
        );

        self.sol_accumulated_deposit_amount = new_sol_accumulated_deposit_amount;
        self.sol_operation_reserved_amount = self
            .sol_operation_reserved_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }

    // TODO visibility is currently set to `in crate::modules` due to operator module - change to `super`
    pub(in crate::modules) fn get_assets_total_amount_as_sol(&self) -> Result<u64> {
        // TODO: need to add the sum(operating sol/tokens) after supported_restaking_protocols add
        self.get_supported_tokens_iter().try_fold(
            self.sol_operation_reserved_amount,
            |sum, token| {
                sum.checked_add(token.get_token_amount_as_sol(token.operation_reserved_amount)?)
                    .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
            },
        )
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct SupportedTokenInfo {
    mint: Pubkey,
    program: Pubkey,
    decimals: u8,
    capacity_amount: u64,
    accumulated_deposit_amount: u64,
    operation_reserved_amount: u64,
    one_token_as_sol: u64,
    pricing_source: TokenPricingSource,
    _reserved: [u8; 128],
}

impl SupportedTokenInfo {
    fn new(
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        capacity_amount: u64,
        pricing_source: TokenPricingSource,
    ) -> Self {
        Self {
            mint,
            program,
            decimals,
            capacity_amount,
            accumulated_deposit_amount: 0,
            operation_reserved_amount: 0,
            one_token_as_sol: 0,
            pricing_source,
            _reserved: [0; 128],
        }
    }

    pub(in crate::modules) fn get_mint(&self) -> Pubkey {
        self.mint
    }

    pub(in crate::modules) fn get_operation_reserved_amount(&self) -> u64 {
        self.operation_reserved_amount
    }

    pub(in crate::modules) fn set_operation_reserved_amount(&mut self, amount: u64) {
        self.operation_reserved_amount = amount;
    }

    fn get_denominated_amount_per_token(&self) -> Result<u64> {
        10u64
            .checked_pow(self.decimals as u32)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }

    #[inline(always)]
    pub(super) fn get_mint_and_pricing_source(&self) -> (Pubkey, TokenPricingSource) {
        (self.mint, self.pricing_source)
    }

    pub(super) fn set_capacity_amount(&mut self, capacity_amount: u64) -> Result<()> {
        require_gte!(
            capacity_amount,
            self.accumulated_deposit_amount,
            ErrorCode::FundInvalidUpdateError
        );

        self.capacity_amount = capacity_amount;

        Ok(())
    }

    pub(super) fn deposit_token(&mut self, amount: u64) -> Result<()> {
        let new_accumulated_deposit_amount = self
            .accumulated_deposit_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        require_gte!(
            self.capacity_amount,
            new_accumulated_deposit_amount,
            ErrorCode::FundExceededTokenCapacityAmountError
        );

        self.accumulated_deposit_amount = new_accumulated_deposit_amount;
        self.operation_reserved_amount = self
            .operation_reserved_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }

    pub(super) fn update_one_token_as_sol(
        &mut self,
        pricing_source_map: &TokenPricingSourceMap,
    ) -> Result<()> {
        self.one_token_as_sol = pricing::calculate_token_amount_as_sol(
            self.mint,
            pricing_source_map,
            self.get_denominated_amount_per_token()?,
        )?;

        Ok(())
    }

    pub(super) fn get_token_amount_as_sol(&self, token_amount: u64) -> Result<u64> {
        crate::utils::get_proportional_amount(
            token_amount,
            self.one_token_as_sol,
            self.get_denominated_amount_per_token()?,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }
}

const MAX_BATCH_WITHDRAWALS_IN_PROGRESS: usize = 10;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct WithdrawalStatus {
    next_batch_id: u64,
    next_request_id: u64,

    num_withdrawal_requests_in_progress: u64,
    last_completed_batch_id: u64,
    last_batch_processing_started_at: Option<i64>,
    last_batch_processing_completed_at: Option<i64>,

    sol_withdrawal_fee_rate: u16,
    withdrawal_enabled_flag: bool,
    batch_processing_threshold_amount: u64,
    batch_processing_threshold_duration: i64,

    // Withdrawal Status = PENDING
    pending_batch_withdrawal: BatchWithdrawal,
    // Withdrawal Status = IN PROGRESS
    // TODO visibility is currently set to `in crate::modules` due to operator module - change to private
    #[max_len(MAX_BATCH_WITHDRAWALS_IN_PROGRESS)]
    pub(in crate::modules) batch_withdrawals_in_progress: Vec<BatchWithdrawal>,
    // Withdrawal Status = COMPLETED
    sol_withdrawal_reserved_amount: u64,
    sol_fee_income_reserved_amount: u64,
    receipt_token_processed_amount: u64,
    _reserved: [u8; 88],
}

impl WithdrawalStatus {
    fn initialize(&mut self) {
        self.next_batch_id = 2;
        self.next_request_id = 1;
        self.num_withdrawal_requests_in_progress = 0;
        self.last_completed_batch_id = 0;
        self.last_batch_processing_started_at = None;
        self.last_batch_processing_completed_at = None;
        self.withdrawal_enabled_flag = true;
        self.sol_withdrawal_fee_rate = 0;
        self.batch_processing_threshold_amount = 0;
        self.batch_processing_threshold_duration = 0;
        self.pending_batch_withdrawal = BatchWithdrawal::new(1);
        self.batch_withdrawals_in_progress = vec![];
        self.sol_withdrawal_reserved_amount = Default::default();
        self.sol_fee_income_reserved_amount = Default::default();
        self.receipt_token_processed_amount = Default::default();
        self._reserved = [0; 88];
    }

    /// 1 fee rate = 1bps = 0.01%
    const WITHDRAWAL_FEE_RATE_DIVISOR: u64 = 10_000;

    #[inline(always)]
    pub(super) fn get_last_completed_batch_id(&self) -> u64 {
        self.last_completed_batch_id
    }

    #[inline(always)]
    pub(super) fn get_sol_withdrawal_fee_rate_as_f32(&self) -> f32 {
        self.sol_withdrawal_fee_rate as f32 / (Self::WITHDRAWAL_FEE_RATE_DIVISOR / 100) as f32
    }

    #[inline(always)]
    pub(super) fn get_withdrawal_enabled_flag(&self) -> bool {
        self.withdrawal_enabled_flag
    }

    #[inline(always)]
    pub(super) fn get_sol_withdrawal_reserved_amount(&self) -> u64 {
        self.sol_withdrawal_reserved_amount
    }

    pub(super) fn set_sol_withdrawal_fee_rate(
        &mut self,
        sol_withdrawal_fee_rate: u16,
    ) -> Result<()> {
        require_gte!(
            Self::WITHDRAWAL_FEE_RATE_DIVISOR,
            sol_withdrawal_fee_rate as u64,
            ErrorCode::FundInvalidSolWithdrawalFeeRateError
        );

        self.sol_withdrawal_fee_rate = sol_withdrawal_fee_rate;

        Ok(())
    }

    #[inline(always)]
    pub(super) fn set_withdrawal_enabled_flag(&mut self, enabled: bool) {
        self.withdrawal_enabled_flag = enabled;
    }

    pub(super) fn set_batch_processing_threshold(
        &mut self,
        amount: Option<u64>,
        duration: Option<i64>,
    ) {
        if let Some(amount) = amount {
            self.batch_processing_threshold_amount = amount;
        }
        if let Some(duration) = duration {
            self.batch_processing_threshold_duration = duration;
        }
    }

    pub(super) fn issue_new_withdrawal_request(
        &mut self,
        receipt_token_amount: u64,
        current_timestamp: i64,
    ) -> Result<WithdrawalRequest> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;

        self.pending_batch_withdrawal
            .add_receipt_token_to_process(receipt_token_amount)?;

        Ok(WithdrawalRequest::new(
            self.pending_batch_withdrawal.batch_id,
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
            self.pending_batch_withdrawal.batch_id,
            ErrorCode::FundProcessingWithdrawalRequestError
        );

        self.pending_batch_withdrawal
            .remove_receipt_token_to_process(request.receipt_token_amount)?;

        Ok(request.receipt_token_amount)
    }

    /// Returns (sol_withdraw_amount, sol_fee_amount, receipt_token_withdraw_amount)
    pub(super) fn claim_withdrawal_request(
        &mut self,
        request: WithdrawalRequest,
    ) -> Result<(u64, u64, u64)> {
        self.assert_request_issued(request.request_id)?;

        require_gte!(
            self.last_completed_batch_id,
            request.batch_id,
            ErrorCode::FundPendingWithdrawalRequestError
        );

        let sol_amount = crate::utils::get_proportional_amount(
            request.receipt_token_amount,
            self.sol_withdrawal_reserved_amount,
            self.receipt_token_processed_amount,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        let sol_fee_amount = crate::utils::get_proportional_amount(
            sol_amount,
            self.sol_withdrawal_fee_rate as u64,
            Self::WITHDRAWAL_FEE_RATE_DIVISOR,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        let sol_transfer_amount = sol_amount
            .checked_sub(sol_fee_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        self.receipt_token_processed_amount = self
            .receipt_token_processed_amount
            .checked_sub(request.receipt_token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self.sol_withdrawal_reserved_amount = self
            .sol_withdrawal_reserved_amount
            .checked_sub(sol_amount)
            .ok_or_else(|| error!(ErrorCode::FundWithdrawalReservedSOLExhaustedException))?;
        self.sol_fee_income_reserved_amount = self
            .sol_fee_income_reserved_amount
            .checked_add(sol_fee_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok((
            sol_transfer_amount,
            sol_fee_amount,
            request.receipt_token_amount,
        ))
    }

    pub(super) fn assert_withdrawal_enabled(&self) -> Result<()> {
        if !self.withdrawal_enabled_flag {
            err!(ErrorCode::FundWithdrawalDisabledError)?
        }

        Ok(())
    }

    // TODO visibility is currently set to `in crate::modules` due to operator module - change to `super`
    pub(in crate::modules) fn assert_withdrawal_threshold_satisfied(
        &self,
        current_timestamp: i64,
    ) -> Result<()> {
        let mut threshold_satisfied = match self.last_batch_processing_started_at {
            Some(x) => current_timestamp - x > self.batch_processing_threshold_duration,
            None => true,
        };

        if self.pending_batch_withdrawal.receipt_token_to_process
            > self.batch_processing_threshold_amount
        {
            threshold_satisfied = true;
        }

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

    // Called by operator
    // TODO visibility is currently set to `in crate::modules` due to operator module - change to `super`
    pub(in crate::modules) fn start_processing_pending_batch_withdrawal(
        &mut self,
        current_timestamp: i64,
    ) -> Result<()> {
        require_gt!(
            MAX_BATCH_WITHDRAWALS_IN_PROGRESS,
            self.batch_withdrawals_in_progress.len(),
            ErrorCode::FundExceededMaxBatchWithdrawalInProgressError
        );

        let batch_id = self.next_batch_id;
        self.next_batch_id += 1;
        let new = BatchWithdrawal::new(batch_id);

        let mut old = std::mem::replace(&mut self.pending_batch_withdrawal, new);
        old.processing_started_at = Some(current_timestamp);

        self.num_withdrawal_requests_in_progress += old.num_withdrawal_requests;
        self.last_batch_processing_started_at = old.processing_started_at;
        self.batch_withdrawals_in_progress.push(old);

        Ok(())
    }

    // Called by operator
    // TODO visibility is currently set to `in crate::modules` due to operator module - change to `super`
    pub(in crate::modules) fn end_processing_completed_batch_withdrawals(
        &mut self,
        current_timestamp: i64,
    ) -> Result<()> {
        let completed_batch_withdrawals = self.pop_completed_batch_withdrawals_from_queue();
        if let Some(batch) = completed_batch_withdrawals.last() {
            self.last_completed_batch_id = batch.batch_id;
            self.last_batch_processing_completed_at = Some(current_timestamp);
        }

        for batch in completed_batch_withdrawals {
            self.receipt_token_processed_amount = self
                .receipt_token_processed_amount
                .checked_add(batch.receipt_token_processed)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
            self.sol_withdrawal_reserved_amount = self
                .sol_withdrawal_reserved_amount
                .checked_add(batch.sol_reserved)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        }

        Ok(())
    }

    fn pop_completed_batch_withdrawals_from_queue(&mut self) -> Vec<BatchWithdrawal> {
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

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize)]
pub struct BatchWithdrawal {
    batch_id: u64,
    num_withdrawal_requests: u64,
    // TODO visibility is currently set to `in crate::modules` due to operator module - change to private
    pub(in crate::modules) receipt_token_to_process: u64,
    // TODO visibility is currently set to `in crate::modules` due to operator module - change to private
    pub(in crate::modules) receipt_token_being_processed: u64,
    receipt_token_processed: u64,
    sol_reserved: u64,
    processing_started_at: Option<i64>,
    _reserved: [u8; 32],
}

impl BatchWithdrawal {
    fn new(batch_id: u64) -> Self {
        Self {
            batch_id,
            num_withdrawal_requests: 0,
            receipt_token_to_process: 0,
            receipt_token_being_processed: 0,
            receipt_token_processed: 0,
            sol_reserved: 0,
            processing_started_at: None,
            _reserved: [0; 32],
        }
    }

    fn add_receipt_token_to_process(&mut self, amount: u64) -> Result<()> {
        self.num_withdrawal_requests += 1;
        self.receipt_token_to_process = self
            .receipt_token_to_process
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }

    pub(super) fn remove_receipt_token_to_process(&mut self, amount: u64) -> Result<()> {
        self.num_withdrawal_requests -= 1;
        self.receipt_token_to_process = self
            .receipt_token_to_process
            .checked_sub(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }

    #[inline(always)]
    fn is_completed(&self) -> bool {
        self.processing_started_at.is_some()
            && self.receipt_token_to_process == 0
            && self.receipt_token_being_processed == 0
    }

    // Called by operator
    // TODO visibility is currently set to `in crate::modules` due to operator module - change to `super`
    pub(in crate::modules) fn record_unstaking_start(
        &mut self,
        receipt_token_amount: u64,
    ) -> Result<()> {
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
    // TODO visibility is currently set to `in crate::modules` due to operator module - change to `super`
    pub(in crate::modules) fn record_unstaking_end(
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

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct WithdrawalRequest {
    batch_id: u64,
    request_id: u64,
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

    #[inline(always)]
    pub(super) fn get_batch_id(&self) -> u64 {
        self.batch_id
    }

    #[inline(always)]
    pub(super) fn get_request_id(&self) -> u64 {
        self.request_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_uninitialized_fund_account() -> FundAccount {
        let buffer = [0u8; 8 + FundAccount::INIT_SPACE];
        FundAccount::try_deserialize_unchecked(&mut &buffer[..]).unwrap()
    }

    #[test]
    fn test_initialize_update_fund_account() {
        let mut fund = create_uninitialized_fund_account();
        fund.initialize(0, Pubkey::new_unique());

        assert_eq!(fund.sol_capacity_amount, 0);
        assert_eq!(fund.withdrawal.sol_withdrawal_fee_rate, 0);
        assert!(fund.withdrawal.withdrawal_enabled_flag);
        assert_eq!(fund.withdrawal.batch_processing_threshold_amount, 0);
        assert_eq!(fund.withdrawal.batch_processing_threshold_duration, 0);

        fund.sol_accumulated_deposit_amount = 1_000_000_000_000;
        fund.set_sol_capacity_amount(0).unwrap_err();

        let new_amount = 10;
        let new_duration = 10;
        fund.withdrawal
            .set_batch_processing_threshold(Some(new_amount), None);
        assert_eq!(
            fund.withdrawal.batch_processing_threshold_amount,
            new_amount
        );
        assert_eq!(fund.withdrawal.batch_processing_threshold_duration, 0);

        fund.withdrawal
            .set_batch_processing_threshold(None, Some(new_duration));
        assert_eq!(
            fund.withdrawal.batch_processing_threshold_amount,
            new_amount
        );
        assert_eq!(
            fund.withdrawal.batch_processing_threshold_duration,
            new_duration
        );
    }

    #[test]
    fn test_update_token() {
        let mut fund = create_uninitialized_fund_account();
        fund.initialize(0, Pubkey::new_unique());

        let token1 = Pubkey::new_unique();
        let token2 = Pubkey::new_unique();

        fund.add_supported_token(
            token1,
            Pubkey::default(),
            9,
            1_000_000_000,
            TokenPricingSource::SPLStakePool {
                address: Pubkey::new_unique(),
            },
        )
        .unwrap();
        fund.add_supported_token(
            token2,
            Pubkey::default(),
            9,
            1_000_000_000,
            TokenPricingSource::MarinadeStakePool {
                address: Pubkey::new_unique(),
            },
        )
        .unwrap();
        fund.add_supported_token(
            token1,
            Pubkey::default(),
            9,
            1_000_000_000,
            TokenPricingSource::MarinadeStakePool {
                address: Pubkey::new_unique(),
            },
        )
        .unwrap_err();
        assert_eq!(fund.supported_tokens.len(), 2);
        assert_eq!(fund.supported_tokens[0].capacity_amount, 1_000_000_000);

        fund.supported_tokens[0].accumulated_deposit_amount = 1_000_000_000;
        fund.get_supported_token_mut(token1)
            .unwrap()
            .set_capacity_amount(0)
            .unwrap_err();
    }

    #[test]
    fn test_deposit_sol() {
        let mut fund = create_uninitialized_fund_account();
        fund.initialize(0, Pubkey::new_unique());
        fund.set_sol_capacity_amount(100_000).unwrap();

        assert_eq!(fund.sol_operation_reserved_amount, 0);
        assert_eq!(fund.sol_accumulated_deposit_amount, 0);

        fund.deposit_sol(100_000).unwrap();
        assert_eq!(fund.sol_operation_reserved_amount, 100_000);
        assert_eq!(fund.sol_accumulated_deposit_amount, 100_000);

        fund.deposit_sol(100_000).unwrap_err();
    }

    #[test]
    fn test_deposit_token() {
        let mut fund = create_uninitialized_fund_account();
        fund.initialize(0, Pubkey::new_unique());

        fund.add_supported_token(
            Pubkey::new_unique(),
            Pubkey::default(),
            9,
            1_000,
            TokenPricingSource::SPLStakePool {
                address: Pubkey::new_unique(),
            },
        )
        .unwrap();

        assert_eq!(fund.supported_tokens[0].operation_reserved_amount, 0);
        assert_eq!(fund.supported_tokens[0].accumulated_deposit_amount, 0);

        fund.supported_tokens[0].deposit_token(1_000).unwrap();
        assert_eq!(fund.supported_tokens[0].operation_reserved_amount, 1_000);
        assert_eq!(fund.supported_tokens[0].accumulated_deposit_amount, 1_000);

        fund.supported_tokens[0].deposit_token(1_000).unwrap_err();
    }
}
