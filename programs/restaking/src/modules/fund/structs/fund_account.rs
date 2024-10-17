use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::utils::PDASeeds;

#[account]
#[derive(InitSpace)]
pub struct FundAccount {
    data_version: u16,
    pub bump: u8,
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
    pub fn initialize_if_needed(&mut self, bump: u8, receipt_token_mint: Pubkey) {
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
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
#[non_exhaustive]
pub enum TokenPricingSource {
    SPLStakePool { address: Pubkey },
    MarinadeStakePool { address: Pubkey },
}

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
    #[max_len(10)]
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

#[cfg(test)]
mod tests {
    use super::*;

    impl FundAccount {
        pub fn new_uninitialized() -> Self {
            Self {
                data_version: 0,
                bump: 0,
                receipt_token_mint: Pubkey::default(),
                supported_tokens: vec![],
                sol_capacity_amount: 0,
                sol_accumulated_deposit_amount: 0,
                sol_operation_reserved_amount: 0,
                withdrawal_status: WithdrawalStatus::new_uninitialized(),
                _reserved: [0; 1280],
            }
        }
    }

    impl WithdrawalStatus {
        fn new_uninitialized() -> Self {
            Self {
                next_batch_id: 0,
                next_request_id: 0,
                num_withdrawal_requests_in_progress: 0,
                last_completed_batch_id: 0,
                last_batch_processing_started_at: None,
                last_batch_processing_completed_at: None,
                sol_withdrawal_fee_rate: 0,
                withdrawal_enabled_flag: false,
                batch_processing_threshold_amount: 0,
                batch_processing_threshold_duration: 0,
                pending_batch_withdrawal: BatchWithdrawal::new(0),
                batch_withdrawals_in_progress: vec![],
                reserved_fund: Default::default(),
            }
        }
    }

    impl SupportedTokenInfo {
        pub fn dummy_spl_stake_pool_token_info(spl_stake_pool_address: Pubkey) -> Self {
            Self {
                mint: Pubkey::new_unique(),
                program: Pubkey::default(),
                decimals: 9,
                capacity_amount: 0,
                accumulated_deposit_amount: 0,
                operation_reserved_amount: 0,
                price: 0,
                pricing_source: TokenPricingSource::SPLStakePool {
                    address: spl_stake_pool_address,
                },
                _reserved: [0; 128],
            }
        }

        pub fn dummy_marinade_stake_pool_token_info(marinade_stake_pool_address: Pubkey) -> Self {
            Self {
                mint: Pubkey::new_unique(),
                program: Pubkey::default(),
                decimals: 9,
                capacity_amount: 0,
                accumulated_deposit_amount: 0,
                operation_reserved_amount: 0,
                price: 0,
                pricing_source: TokenPricingSource::MarinadeStakePool {
                    address: marinade_stake_pool_address,
                },
                _reserved: [0; 128],
            }
        }
    }

    #[test]
    fn test_initialize_fund_account() {
        let receipt_token_mint = Pubkey::new_unique();
        let mut fund = FundAccount::new_uninitialized();

        assert_eq!(fund.withdrawal_status.next_batch_id, 0);
        assert_eq!(fund.withdrawal_status.next_request_id, 0);
        assert!(!fund.withdrawal_status.withdrawal_enabled_flag);
        assert_eq!(fund.withdrawal_status.pending_batch_withdrawal.batch_id, 0);

        fund.initialize_if_needed(0, receipt_token_mint);

        assert_eq!(fund.withdrawal_status.next_batch_id, 2);
        assert_eq!(fund.withdrawal_status.next_request_id, 1);
        assert!(fund.withdrawal_status.withdrawal_enabled_flag);
        assert_eq!(fund.withdrawal_status.pending_batch_withdrawal.batch_id, 1);
        assert_eq!(
            fund.withdrawal_status
                .pending_batch_withdrawal
                .num_withdrawal_requests,
            0
        );
        assert_eq!(
            fund.withdrawal_status
                .pending_batch_withdrawal
                .receipt_token_to_process,
            0
        );
    }
}
