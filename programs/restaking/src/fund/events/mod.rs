use anchor_lang::prelude::*;

use crate::TokenInfo;

#[event]
pub struct FundInfoEvent {
    pub admin: Pubkey,
    pub receipt_token_mint: Pubkey,
    pub default_token_fee_rate: u16,
    pub whitelisted_tokens: Vec<TokenInfo>,
    pub sol_amount_in: u128,
}

/// Split into `FundDepositSOLEvent` / `FundDepositTokenEvent`??
#[event]
pub struct FundDepositEvent {
    /// redundant info?
    pub is_token: bool,
    pub token_mint: Option<Pubkey>,
    pub receipt_token_mint: Pubkey,
    pub sol_deposit_amount: u64,
    pub sol_amount_in_fund: u128,
    pub token_deposit_amount: Option<u64>,
    pub token_amount_in_fund: Option<u128>,
    pub receipt_token_mint_amount: u64,
}

/// When user requested withdrawal
#[event]
pub struct FundRequestWithdrawalEvent {
    /// `user` vs. `user_wallet`??
    pub user_wallet: Pubkey,
    pub user_account: Pubkey,
    pub receipt_token_mint: Pubkey,
    pub batch_id: u64,
    pub request_id: u64,
    pub receipt_token_amount: u64,
    pub timestamp: i64,
}

/// When user withdrawed SOL
#[event]
pub struct FundWithdrawEvent {
    pub user_wallet: Pubkey,
    pub user_account: Pubkey,
    pub receipt_token_mint: Pubkey,
    pub batch_id: u64,
    pub request_id: u64,
    pub receipt_token_amount: u64,
    pub sol_withdraw_amount: u64,
    pub sol_remaining_in_reserved_fund: u128,
}

/// When operator started processing pending batch
///
/// Can we merge `FundStartBatchWithdrawalEvent`, `FundProcessBatchWithdrawalEvent`,
/// and `FundEndBatchWithdrawalEvent` into a single event??
#[event]
pub struct FundStartBatchWithdrawalEvent {
    pub receipt_token_mint: Pubkey,
    pub batch_id: u64,
    pub num_withdrawal_requests: u64,
    pub receipt_token_to_process: u128,
}

/// When operator partially processed the batch in progress
#[event]
pub struct FundProcessBatchWithdrawalEvent {
    pub receipt_token_mint: Pubkey,
    pub batch_id: u64,
    pub num_withdrawal_requests: u64,
    pub receipt_token_to_process: u128,
    pub receipt_token_being_processed: u128,
    pub receipt_token_processed: u128,
    pub sol_reserved: u128,
}

/// When operator ended processing the batch in progress
#[event]
pub struct FundEndBatchWithdrawalEvent {
    pub receipt_token_mint: Pubkey,
    pub batch_id: u64,
    pub num_withdrawal_requests_processed: u64,
    pub total_withdrawal_requests_processed: u64,
    pub receipt_token_processed: u128,
    pub total_receipt_token_processed: u128,
    pub sol_reserved: u128,
    pub total_sol_reserved: u128,
    pub sol_remaining: u128,
}
