mod info;
mod sol_deposited;
mod sol_withdrawed;
mod token_deposited;
mod withdrawal_request_cancelled;
mod withdrawal_requested;

pub use info::*;
pub use sol_deposited::*;
pub use sol_withdrawed::*;
pub use token_deposited::*;
pub use withdrawal_request_cancelled::*;
pub use withdrawal_requested::*;

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
