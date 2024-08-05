use anchor_lang::prelude::*;

use crate::TokenInfo;

#[event]
pub struct FundInfo {
    pub admin: Pubkey,
    pub lrt_mint: Pubkey,
    pub supported_tokens: Vec<TokenInfo>,
    pub sol_amount_in: u128,
    pub sol_reserved_amount: u128,
    pub sol_withdrawal_fee_rate: f32,
    pub sol_withdrawal_enabled: bool,
}

#[event]
pub struct FundSOLDeposited {
    pub user: Pubkey,
    pub user_minted_lrt_account: Pubkey,

    pub sol_deposit_amount: u64,
    pub sol_amount_in_fund: u128,
    pub minted_lrt_mint: Pubkey,
    pub minted_lrt_amount: u64,

    // Not Implemented Yet: Always `None` for now
    pub wallet_provider: Option<String>,
    pub fpoint_accrual_rate_multiplier: Option<f32>,

    pub fund_info: FundInfo,
}

#[event]
pub struct FundTokenDeposited {
    pub user: Pubkey,
    pub user_minted_lrt_account: Pubkey,

    pub deposited_token_mint: Pubkey,
    pub deposited_token_user_account: Pubkey,

    pub token_deposit_amount: u64,
    pub token_amount_in_fund: u128,
    pub minted_lrt_mint: Pubkey,
    pub minted_lrt_amount: u64,

    // Not Implemented Yet: Always `None` for now
    pub wallet_provider: Option<String>,
    pub fpoint_accrual_rate_multiplier: Option<f32>,

    pub fund_info: FundInfo,
}

#[event]
pub struct LRTTransferred {
    pub lrt_mint: Pubkey,
    pub lrt_amount: Pubkey,
    pub source_lrt_account: Pubkey,
    pub destination_lrt_account: Pubkey,
}

#[event]
pub struct FundWithdrawalRequested {
    pub user: Pubkey,
    pub user_receipt_account: Pubkey,   // Receipt of withdrawal request

    pub request_id: u64,
    pub lrt_mint: Pubkey,
    pub lrt_amount: u64,
}

#[event]
pub struct FundWithdrawalRequestCancelled {
    pub user: Pubkey,
    pub user_receipt_account: Pubkey,   // Receipt of withdrawal request
    
    pub request_id: u64,
    pub lrt_mint: Pubkey,
    pub lrt_amount: u64,
}

#[event]
pub struct FundSOLWithdrawed {
    pub user: Pubkey,
    pub user_receipt_account: Pubkey,   // Receipt of withdrawal request
    pub user_minted_lrt_account: Pubkey,

    pub request_id: u64,
    pub lrt_mint: Pubkey,
    pub lrt_amount: u64,

    pub sol_withdraw_amount: u64,
    pub sol_fee_amount: u64,

    pub fund_info: FundInfo,
}

// // TO BE DONE
// /// When operator started processing pending batch
// ///
// /// Can we merge `FundStartBatchWithdrawalEvent`, `FundProcessBatchWithdrawalEvent`,
// /// and `FundEndBatchWithdrawalEvent` into a single event??
// #[event]
// pub struct FundStartBatchWithdrawalEvent {
//     pub receipt_token_mint: Pubkey,
//     pub batch_id: u64,
//     pub num_withdrawal_requests: u64,
//     pub receipt_token_to_process: u128,
// }

// /// When operator partially processed the batch in progress
// #[event]
// pub struct FundProcessBatchWithdrawalEvent {
//     pub receipt_token_mint: Pubkey,
//     pub batch_id: u64,
//     pub num_withdrawal_requests: u64,
//     pub receipt_token_to_process: u128,
//     pub receipt_token_being_processed: u128,
//     pub receipt_token_processed: u128,
//     pub sol_reserved: u128,
// }

// /// When operator ended processing the batch in progress
// #[event]
// pub struct FundEndBatchWithdrawalEvent {
//     pub receipt_token_mint: Pubkey,
//     pub batch_id: u64,
//     pub num_withdrawal_requests_processed: u64,
//     pub total_withdrawal_requests_processed: u64,
//     pub receipt_token_processed: u128,
//     pub total_receipt_token_processed: u128,
//     pub sol_reserved: u128,
//     pub total_sol_reserved: u128,
//     pub sol_remaining: u128,
// }