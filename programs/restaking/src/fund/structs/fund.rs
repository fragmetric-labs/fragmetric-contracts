use anchor_lang::prelude::*;
use fragmetric_util::{RequireUpgradable, Upgradable};

#[account]
#[derive(InitSpace, RequireUpgradable)]
pub struct Fund {
    pub admin: Pubkey,
    pub receipt_token_mint: Pubkey,
    #[upgradable(latest = FundV2, variant = V2)]
    pub data: VersionedFund,
}

impl Upgradable for Fund {
    type LatestVersion = FundV2;

    fn upgrade(&mut self) {
        match self.data {
            VersionedFund::V1(ref mut old) => {
                let whitelisted_tokens = std::mem::take(&mut old.whitelisted_tokens);
                self.data = VersionedFund::V2(FundV2 {
                    whitelisted_tokens,
                    sol_amount_in: old.sol_amount_in,
                    withdrawal_status: WithdrawalStatus {
                        sol_withdrawal_fee_rate: old.sol_withdrawal_fee_rate,
                        ..Default::default()
                    },
                });
            }
            VersionedFund::V2(_) => (),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub enum VersionedFund {
    V1(FundV1),
    V2(FundV2),
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct FundV1 {
    pub sol_withdrawal_fee_rate: u16, // 2
    #[max_len(20)]
    pub whitelisted_tokens: Vec<TokenInfo>,
    pub sol_amount_in: u128, // 16
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct FundV2 {
    #[max_len(20)]
    pub whitelisted_tokens: Vec<TokenInfo>,
    pub sol_amount_in: u128, // 16
    pub withdrawal_status: WithdrawalStatus,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct TokenInfo {
    pub address: Pubkey,
    pub token_cap: u128,
    pub token_amount_in: u128,
}

impl TokenInfo {
    pub fn empty(address: Pubkey, token_cap: u128) -> Self {
        Self {
            address,
            token_cap,
            token_amount_in: 0,
        }
    }
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
    pub batch_processing_threshold_amount: u128,
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

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize)]
pub struct BatchWithdrawal {
    pub batch_id: u64,
    pub num_withdrawal_requests: u64,
    pub receipt_token_to_process: u128,
    pub receipt_token_being_processed: u128,
    pub receipt_token_processed: u128,
    pub sol_reserved: u128,
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
    pub sol_remaining: u128,
}
