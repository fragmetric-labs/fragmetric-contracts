use anchor_lang::prelude::*;

use crate::PDASignerSeeds;

#[account]
#[derive(InitSpace)]
pub struct Fund {
    pub data_version: u8,
    pub bump: u8,
    pub receipt_token_mint: Pubkey,
    #[max_len(20)]
    pub supported_tokens: Vec<SupportedTokenInfo>,
    pub sol_operation_reserved_amount: u64,
    pub withdrawal_status: WithdrawalStatus,
}

impl PDASignerSeeds<3> for Fund {
    const SEED: &'static [u8] = b"fund_seed";

    fn signer_seeds(&self) -> [&[u8]; 3] {
        [
            Self::SEED,
            self.receipt_token_mint.as_ref(),
            self.bump_as_slice(),
        ]
    }

    fn bump_ref(&self) -> &u8 {
        &self.bump
    }
}

impl Fund {
    pub fn supported_token_position(&self, token: Pubkey) -> Option<usize> {
        self.supported_tokens
            .iter()
            .position(|info| info.mint == token)
    }

    pub fn supported_token_mut(&mut self, token: Pubkey) -> Option<&mut SupportedTokenInfo> {
        self.supported_tokens
            .iter_mut()
            .find(|info| info.mint == token)
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct SupportedTokenInfo {
    pub mint: Pubkey,
    pub decimals: u8,
    pub capacity_amount: u64,
    pub operation_reserved_amount: u64,
    pub price: u64,
    pub pricing_source: TokenPricingSource,
}

impl SupportedTokenInfo {
    pub fn empty(
        mint: Pubkey,
        decimals: u8,
        capacity_amount: u64,
        pricing_source: TokenPricingSource,
    ) -> Self {
        Self {
            mint,
            decimals,
            capacity_amount,
            operation_reserved_amount: 0,
            price: 0,
            pricing_source,
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
            pending_batch_withdrawal: BatchWithdrawal::empty(1),
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
}

impl BatchWithdrawal {
    pub fn empty(batch_id: u64) -> Self {
        Self {
            batch_id,
            num_withdrawal_requests: 0,
            receipt_token_to_process: 0,
            receipt_token_being_processed: 0,
            receipt_token_processed: 0,
            sol_reserved: 0,
            processing_started_at: None,
        }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Default)]
pub struct ReservedFund {
    pub num_completed_withdrawal_requests: u64,
    pub total_receipt_token_processed: u128,
    pub total_sol_reserved: u128,
    pub sol_remaining: u64,
}
