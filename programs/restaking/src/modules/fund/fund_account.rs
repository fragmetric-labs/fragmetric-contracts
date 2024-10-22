use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::price::TokenPricingSource;
use crate::utils::PDASeeds;

#[constant]
/// ## Version History
/// * v1: Initial Version
/// * v2: Change reserve fund structure
pub const FUND_ACCOUNT_CURRENT_VERSION: u16 = 2;

#[account]
#[derive(InitSpace)]
pub struct FundAccount {
    data_version: u16,
    bump: u8,
    pub receipt_token_mint: Pubkey,
    #[max_len(16)]
    pub supported_tokens: Vec<SupportedTokenInfo>,
    pub sol_capacity_amount: u64,
    pub sol_accumulated_deposit_amount: u64,
    pub sol_operation_reserved_amount: u64,
    pub withdrawal_status: WithdrawalStatus,
    pub _reserved: [u8; 1280],
}

impl PDASeeds<2> for FundAccount {
    const SEED: &'static [u8] = b"fund";

    fn seeds(&self) -> [&[u8]; 2] {
        [Self::SEED, self.receipt_token_mint.as_ref()]
    }

    fn bump_ref(&self) -> &u8 {
        &self.bump
    }
}

impl FundAccount {
    pub fn initialize(&mut self, bump: u8, receipt_token_mint: Pubkey) {
        if self.data_version == 0 {
            self.data_version = 1;
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
            self.withdrawal_status = Default::default();
        }

        if self.data_version == 1 {
            self.data_version = 2;
            self.withdrawal_status
                .reserved_fund
                .sol_withdrawal_reserved_amount = 0;
            self.withdrawal_status
                .reserved_fund
                .receipt_token_processed_amount = 0;
            self.withdrawal_status.reserved_fund._reserved = [0; 88];
        }
    }

    pub fn update_if_needed(&mut self, receipt_token_mint: Pubkey) {
        self.initialize(self.bump, receipt_token_mint);
    }

    pub fn is_latest_version(&self) -> bool {
        self.data_version == FUND_ACCOUNT_CURRENT_VERSION
    }

    pub fn supported_token(&self, token: Pubkey) -> Result<&SupportedTokenInfo> {
        self.supported_tokens
            .iter()
            .find(|info| info.mint == token)
            .ok_or_else(|| error!(ErrorCode::FundNotSupportedTokenError))
    }

    pub fn supported_token_mut(&mut self, token: Pubkey) -> Result<&mut SupportedTokenInfo> {
        self.supported_tokens
            .iter_mut()
            .find(|info| info.mint == token)
            .ok_or_else(|| error!(ErrorCode::FundNotSupportedTokenError))
    }

    pub fn set_sol_capacity_amount(&mut self, capacity_amount: u64) -> Result<()> {
        require_gte!(
            capacity_amount,
            self.sol_accumulated_deposit_amount,
            ErrorCode::FundInvalidUpdateError
        );

        self.sol_capacity_amount = capacity_amount;

        Ok(())
    }

    pub fn add_supported_token(
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
        let token_info =
            SupportedTokenInfo::new(mint, program, decimals, capacity_amount, pricing_source);
        self.supported_tokens.push(token_info);

        Ok(())
    }

    pub fn deposit_sol(&mut self, amount: u64) -> Result<()> {
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
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct SupportedTokenInfo {
    pub mint: Pubkey,
    pub program: Pubkey,
    pub decimals: u8,
    pub capacity_amount: u64,
    pub accumulated_deposit_amount: u64,
    pub operation_reserved_amount: u64,
    pub price: u64,
    pub pricing_source: TokenPricingSource,
    pub _reserved: [u8; 128],
}

impl SupportedTokenInfo {
    pub fn new(
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
            price: 0,
            pricing_source,
            _reserved: [0; 128],
        }
    }

    pub fn set_capacity_amount(&mut self, capacity_amount: u64) -> Result<()> {
        require_gte!(
            capacity_amount,
            self.accumulated_deposit_amount,
            ErrorCode::FundInvalidUpdateError
        );

        self.capacity_amount = capacity_amount;

        Ok(())
    }

    pub fn deposit_token(&mut self, amount: u64) -> Result<()> {
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
}

const MAX_BATCH_WITHDRAWALS_IN_PROGRESS: usize = 10;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct WithdrawalStatus {
    pub next_batch_id: u64,
    pub next_request_id: u64,

    pub num_withdrawal_requests_in_progress: u64,
    pub last_completed_batch_id: u64,
    pub last_batch_processing_started_at: Option<i64>,
    pub last_batch_processing_completed_at: Option<i64>,

    pub sol_withdrawal_fee_rate: u16,
    pub withdrawal_enabled_flag: bool,
    pub batch_processing_threshold_amount: u64,
    pub batch_processing_threshold_duration: i64,

    // Withdrawal Status = PENDING
    pub pending_batch_withdrawal: BatchWithdrawal,
    // Withdrawal Status = IN PROGRESS
    #[max_len(MAX_BATCH_WITHDRAWALS_IN_PROGRESS)]
    pub batch_withdrawals_in_progress: Vec<BatchWithdrawal>,
    // Withdrawal Status = COMPLETED
    pub reserved_fund: ReservedFund,
}

impl Default for WithdrawalStatus {
    fn default() -> Self {
        Self {
            next_batch_id: 2,
            next_request_id: 1,
            num_withdrawal_requests_in_progress: 0,
            last_completed_batch_id: 0,
            last_batch_processing_started_at: None,
            last_batch_processing_completed_at: None,
            withdrawal_enabled_flag: true,
            sol_withdrawal_fee_rate: 0,
            batch_processing_threshold_amount: 0,
            batch_processing_threshold_duration: 0,
            pending_batch_withdrawal: BatchWithdrawal::new(1),
            batch_withdrawals_in_progress: vec![],
            reserved_fund: Default::default(),
        }
    }
}

impl WithdrawalStatus {
    /// 1 fee rate = 1bps = 0.01%
    pub const WITHDRAWAL_FEE_RATE_DIVISOR: u64 = 10_000;

    pub fn sol_withdrawal_fee_rate_f32(&self) -> f32 {
        self.sol_withdrawal_fee_rate as f32 / (Self::WITHDRAWAL_FEE_RATE_DIVISOR / 100) as f32
    }

    pub fn set_sol_withdrawal_fee_rate(&mut self, sol_withdrawal_fee_rate: u16) {
        self.sol_withdrawal_fee_rate = sol_withdrawal_fee_rate;
    }

    pub fn set_withdrawal_enabled_flag(&mut self, flag: bool) {
        self.withdrawal_enabled_flag = flag;
    }

    pub fn set_batch_processing_threshold(&mut self, amount: Option<u64>, duration: Option<i64>) {
        if let Some(amount) = amount {
            self.batch_processing_threshold_amount = amount;
        }
        if let Some(duration) = duration {
            self.batch_processing_threshold_duration = duration;
        }
    }

    pub(super) fn issue_new_request_id(&mut self) -> u64 {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        request_id
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
        receipt_token_amount: u64,
    ) -> Result<()> {
        self.reserved_fund
            .withdraw(sol_amount, sol_fee_amount, receipt_token_amount)
    }

    pub(super) fn check_withdrawal_enabled(&self) -> Result<()> {
        if !self.withdrawal_enabled_flag {
            err!(ErrorCode::FundWithdrawalDisabledError)?
        }

        Ok(())
    }

    pub fn check_withdrawal_threshold(&self, current_time: i64) -> Result<()> {
        let mut threshold_satisfied = match self.last_batch_processing_started_at {
            Some(x) => current_time - x > self.batch_processing_threshold_duration,
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

    pub(super) fn check_batch_processing_not_started(&self, batch_id: u64) -> Result<()> {
        require_gte!(
            batch_id,
            self.pending_batch_withdrawal.batch_id,
            ErrorCode::FundProcessingWithdrawalRequestError
        );

        Ok(())
    }

    pub(super) fn check_batch_processing_completed(&self, batch_id: u64) -> Result<()> {
        require_gte!(
            self.last_completed_batch_id,
            batch_id,
            ErrorCode::FundPendingWithdrawalRequestError
        );

        Ok(())
    }

    // Called by operator
    pub fn start_processing_pending_batch_withdrawal(&mut self, current_time: i64) -> Result<()> {
        require_gt!(
            MAX_BATCH_WITHDRAWALS_IN_PROGRESS,
            self.batch_withdrawals_in_progress.len(),
            ErrorCode::FundExceededMaxBatchWithdrawalInProgressError
        );

        let batch_id = self.next_batch_id;
        self.next_batch_id += 1;
        let new = BatchWithdrawal::new(batch_id);

        let mut old = std::mem::replace(&mut self.pending_batch_withdrawal, new);
        old.start_batch_processing(current_time);

        self.num_withdrawal_requests_in_progress += old.num_withdrawal_requests;
        self.last_batch_processing_started_at = old.processing_started_at;
        self.batch_withdrawals_in_progress.push(old);

        Ok(())
    }

    // Called by operator
    pub fn end_processing_completed_batch_withdrawals(&mut self, current_time: i64) -> Result<()> {
        let completed_batch_withdrawals = self.pop_completed_batch_withdrawals();
        if let Some(batch) = completed_batch_withdrawals.last() {
            self.last_completed_batch_id = batch.batch_id;
            self.last_batch_processing_completed_at = Some(current_time);
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

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize)]
pub struct BatchWithdrawal {
    pub batch_id: u64,
    pub num_withdrawal_requests: u64,
    pub receipt_token_to_process: u64,
    pub receipt_token_being_processed: u64,
    pub receipt_token_processed: u64,
    pub sol_reserved: u64,
    pub processing_started_at: Option<i64>,
    pub _reserved: [u8; 32],
}

impl BatchWithdrawal {
    pub fn new(batch_id: u64) -> Self {
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

    pub(super) fn add_receipt_token_to_process(&mut self, amount: u64) -> Result<()> {
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

    fn start_batch_processing(&mut self, current_time: i64) {
        self.processing_started_at = Some(current_time);
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

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize)]
pub struct ReservedFund {
    pub sol_withdrawal_reserved_amount: u64,
    pub sol_fee_income_reserved_amount: u64,
    pub receipt_token_processed_amount: u64,
    pub _reserved: [u8; 88],
}

impl Default for ReservedFund {
    fn default() -> Self {
        Self {
            sol_withdrawal_reserved_amount: Default::default(),
            sol_fee_income_reserved_amount: Default::default(),
            receipt_token_processed_amount: Default::default(),
            _reserved: [0; 88],
        }
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
        receipt_token_amount: u64,
    ) -> Result<()> {
        self.receipt_token_processed_amount = self
            .receipt_token_processed_amount
            .checked_sub(receipt_token_amount)
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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     impl FundAccount {
//         pub fn new_uninitialized() -> Self {
//             Self {
//                 data_version: 0,
//                 bump: 0,
//                 receipt_token_mint: Pubkey::default(),
//                 supported_tokens: vec![],
//                 sol_capacity_amount: 0,
//                 sol_accumulated_deposit_amount: 0,
//                 sol_operation_reserved_amount: 0,
//                 withdrawal_status: WithdrawalStatus::new_uninitialized(),
//                 _reserved: [0; 1280],
//             }
//         }
//     }

//     impl WithdrawalStatus {
//         fn new_uninitialized() -> Self {
//             Self {
//                 next_batch_id: 0,
//                 next_request_id: 0,
//                 num_withdrawal_requests_in_progress: 0,
//                 last_completed_batch_id: 0,
//                 last_batch_processing_started_at: None,
//                 last_batch_processing_completed_at: None,
//                 sol_withdrawal_fee_rate: 0,
//                 withdrawal_enabled_flag: false,
//                 batch_processing_threshold_amount: 0,
//                 batch_processing_threshold_duration: 0,
//                 pending_batch_withdrawal: BatchWithdrawal::new(0),
//                 batch_withdrawals_in_progress: vec![],
//                 reserved_fund: Default::default(),
//             }
//         }
//     }

//     impl SupportedTokenInfo {
//         pub fn dummy_spl_stake_pool_token_info(spl_stake_pool_address: Pubkey) -> Self {
//             Self {
//                 mint: Pubkey::new_unique(),
//                 program: Pubkey::default(),
//                 decimals: 9,
//                 capacity_amount: 0,
//                 accumulated_deposit_amount: 0,
//                 operation_reserved_amount: 0,
//                 price: 0,
//                 pricing_source: TokenPricingSource::SPLStakePool {
//                     address: spl_stake_pool_address,
//                 },
//                 _reserved: [0; 128],
//             }
//         }

//         pub fn dummy_marinade_stake_pool_token_info(marinade_stake_pool_address: Pubkey) -> Self {
//             Self {
//                 mint: Pubkey::new_unique(),
//                 program: Pubkey::default(),
//                 decimals: 9,
//                 capacity_amount: 0,
//                 accumulated_deposit_amount: 0,
//                 operation_reserved_amount: 0,
//                 price: 0,
//                 pricing_source: TokenPricingSource::MarinadeStakePool {
//                     address: marinade_stake_pool_address,
//                 },
//                 _reserved: [0; 128],
//             }
//         }
//     }

//     #[test]
//     fn test_initialize_fund_account() {
//         let receipt_token_mint = Pubkey::new_unique();
//         let mut fund = FundAccount::new_uninitialized();

//         assert_eq!(fund.withdrawal_status.next_batch_id, 0);
//         assert_eq!(fund.withdrawal_status.next_request_id, 0);
//         assert!(!fund.withdrawal_status.withdrawal_enabled_flag);
//         assert_eq!(fund.withdrawal_status.pending_batch_withdrawal.batch_id, 0);

//         fund.initialize(0, receipt_token_mint);

//         assert_eq!(fund.withdrawal_status.next_batch_id, 2);
//         assert_eq!(fund.withdrawal_status.next_request_id, 1);
//         assert!(fund.withdrawal_status.withdrawal_enabled_flag);
//         assert_eq!(fund.withdrawal_status.pending_batch_withdrawal.batch_id, 1);
//         assert_eq!(
//             fund.withdrawal_status
//                 .pending_batch_withdrawal
//                 .num_withdrawal_requests,
//             0
//         );
//         assert_eq!(
//             fund.withdrawal_status
//                 .pending_batch_withdrawal
//                 .receipt_token_to_process,
//             0
//         );
//     }

//     #[test]
//     fn test_update_fund() {
//         let mut fund = FundAccount::new_uninitialized();
//         fund.initialize(0, Pubkey::new_unique());

//         assert_eq!(fund.sol_capacity_amount, 0);
//         assert_eq!(fund.withdrawal_status.sol_withdrawal_fee_rate, 0);
//         assert!(fund.withdrawal_status.withdrawal_enabled_flag);
//         assert_eq!(fund.withdrawal_status.batch_processing_threshold_amount, 0);
//         assert_eq!(
//             fund.withdrawal_status.batch_processing_threshold_duration,
//             0
//         );

//         let new_sol_capacity_amount = 1_000_000_000 * 60_000;
//         fund.set_sol_capacity_amount(new_sol_capacity_amount)
//             .unwrap();
//         assert_eq!(fund.sol_capacity_amount, new_sol_capacity_amount);

//         let new_sol_withdrawal_fee_rate = 20;
//         fund.withdrawal_status
//             .set_sol_withdrawal_fee_rate(new_sol_withdrawal_fee_rate);
//         assert_eq!(
//             fund.withdrawal_status.sol_withdrawal_fee_rate,
//             new_sol_withdrawal_fee_rate
//         );

//         fund.withdrawal_status.set_withdrawal_enabled_flag(false);
//         assert!(!fund.withdrawal_status.withdrawal_enabled_flag);

//         let new_amount = 10;
//         let new_duration = 10;
//         fund.withdrawal_status
//             .set_batch_processing_threshold(Some(new_amount), None);
//         assert_eq!(
//             fund.withdrawal_status.batch_processing_threshold_amount,
//             new_amount
//         );
//         assert_eq!(
//             fund.withdrawal_status.batch_processing_threshold_duration,
//             0
//         );

//         fund.withdrawal_status
//             .set_batch_processing_threshold(None, Some(new_duration));
//         assert_eq!(
//             fund.withdrawal_status.batch_processing_threshold_amount,
//             new_amount
//         );
//         assert_eq!(
//             fund.withdrawal_status.batch_processing_threshold_duration,
//             new_duration
//         );
//     }

//     #[test]
//     fn test_update_token() {
//         let mut fund = FundAccount::new_uninitialized();
//         fund.initialize(0, Pubkey::new_unique());

//         let mut dummy_lamports = 0u64;
//         let mut dummy_data = [0u8; std::mem::size_of::<SplStakePool>()];
//         let mut dummy_lamports2 = 0u64;
//         let mut dummy_data2 = [0u8; 8 + MarinadeStakePool::INIT_SPACE];
//         let pricing_sources = &[
//             SplStakePool::placeholder(&mut dummy_lamports, &mut dummy_data),
//             MarinadeStakePool::placeholder(&mut dummy_lamports2, &mut dummy_data2),
//         ];
//         let token1 = SupportedTokenInfo::dummy_spl_stake_pool_token_info(pricing_sources[0].key());
//         let token2 =
//             SupportedTokenInfo::dummy_marinade_stake_pool_token_info(pricing_sources[1].key());

//         fund.add_supported_token(
//             token1.mint,
//             token1.program,
//             token1.decimals,
//             token1.capacity_amount,
//             token1.pricing_source,
//         )
//         .unwrap();
//         fund.add_supported_token(
//             token2.mint,
//             token2.program,
//             token2.decimals,
//             token2.capacity_amount,
//             token2.pricing_source,
//         )
//         .unwrap();
//         assert_eq!(fund.supported_tokens.len(), 2);
//         assert_eq!(
//             fund.supported_tokens[0].capacity_amount,
//             token1.capacity_amount
//         );

//         let new_token1_capacity_amount = 1_000_000_000 * 3000;
//         fund.supported_token_mut(token1.mint)
//             .unwrap()
//             .set_capacity_amount(new_token1_capacity_amount)
//             .unwrap();
//         assert_eq!(
//             fund.supported_tokens[0].capacity_amount,
//             new_token1_capacity_amount
//         );
//     }

//     #[test]
//     fn test_deposit_sol() {
//         let mut fund = FundAccount::new_uninitialized();
//         fund.initialize(0, Pubkey::new_unique());
//         fund.set_sol_capacity_amount(100_000).unwrap();

//         assert_eq!(fund.sol_operation_reserved_amount, 0);
//         assert_eq!(fund.sol_accumulated_deposit_amount, 0);

//         fund.deposit_sol(100_000).unwrap();
//         assert_eq!(fund.sol_operation_reserved_amount, 100_000);
//         assert_eq!(fund.sol_accumulated_deposit_amount, 100_000);

//         fund.deposit_sol(100_000).unwrap_err();
//     }

//     #[test]
//     fn test_deposit_token() {
//         let mut fund = FundAccount::new_uninitialized();
//         fund.initialize(0, Pubkey::new_unique());

//         let mut dummy_lamports = 0u64;
//         let mut dummy_data = [0u8; std::mem::size_of::<SplStakePool>()];
//         let pricing_sources = &[SplStakePool::placeholder(
//             &mut dummy_lamports,
//             &mut dummy_data,
//         )];
//         let token = SupportedTokenInfo::dummy_spl_stake_pool_token_info(pricing_sources[0].key());

//         fund.add_supported_token(
//             token.mint,
//             token.program,
//             token.decimals,
//             1_000,
//             token.pricing_source,
//         )
//         .unwrap();

//         assert_eq!(fund.supported_tokens[0].operation_reserved_amount, 0);
//         assert_eq!(fund.supported_tokens[0].accumulated_deposit_amount, 0);

//         fund.supported_tokens[0].deposit_token(1_000).unwrap();
//         assert_eq!(fund.supported_tokens[0].operation_reserved_amount, 1_000);
//         assert_eq!(fund.supported_tokens[0].accumulated_deposit_amount, 1_000);

//         fund.supported_tokens[0].deposit_token(1_000).unwrap_err();
//     }
// }
