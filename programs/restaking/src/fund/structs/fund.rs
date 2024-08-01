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
                    default_protocol_fee_rate: old.default_protocol_fee_rate,
                    whitelisted_tokens,
                    sol_amount_in: old.sol_amount_in,
                    withdrawal_enabled_flag: true,
                    pending_withdrawals: BatchWithdrawal::new(1),
                    withdrawals_in_progress: Default::default(),
                    reserved_fund: Default::default(),
                });
            }
            VersionedFund::V2(_) => (),
        }
    }
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub enum VersionedFund {
    V1(FundV1),
    V2(FundV2),
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct FundV1 {
    pub default_protocol_fee_rate: u16, // 2
    #[max_len(20)]
    pub whitelisted_tokens: Vec<TokenInfo>,
    pub sol_amount_in: u128, // 16
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct FundV2 {
    pub default_protocol_fee_rate: u16, // 2
    #[max_len(20)]
    pub whitelisted_tokens: Vec<TokenInfo>,
    pub sol_amount_in: u128, // 16
    pub withdrawal_enabled_flag: bool,
    // Withdrawal Status = PENDING
    pub pending_withdrawals: BatchWithdrawal,
    // Withdrawal Status = IN PROGRESS
    pub withdrawals_in_progress: WithdrawalsInProgress,
    // Withdrawal Status = COMPLETED
    pub reserved_fund: ReservedFund,
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

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize)]
pub struct BatchWithdrawal {
    pub batch_id: u64,
    pub num_withdrawal_requests: u64,
    pub receipt_token_to_process: u128,
    pub receipt_token_being_processed: u128,
    pub receipt_token_processed: u128,
    pub sol_reserved: u128,
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
        }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Default)]
pub struct WithdrawalsInProgress {
    pub num_withdrawal_requests_in_progress: u64,
    #[max_len(10)]
    pub batch_withdrawal_queue: Vec<BatchWithdrawal>,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Default)]
pub struct ReservedFund {
    pub last_completed_batch_id: u64,
    pub num_completed_withdrawal_requests: u64,
    pub total_receipt_token_processed: u128,
    pub total_sol_reserved: u128,
    pub sol_remaining: u128,
}
