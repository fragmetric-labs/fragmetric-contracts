mod info;
mod price_updated;
mod sol_deposited;
mod sol_withdrawn;
mod token_deposited;
mod withdrawal_request_canceled;
mod withdrawal_requested;

pub use info::*;
pub use price_updated::*;
pub use sol_deposited::*;
pub use sol_withdrawn::*;
pub use token_deposited::*;
pub use withdrawal_request_canceled::*;
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
//     pub receipt_token_to_process: u64,
// }

// /// When operator partially processed the batch in progress
// #[event]
// pub struct FundProcessBatchWithdrawalEvent {
//     pub receipt_token_mint: Pubkey,
//     pub batch_id: u64,
//     pub num_withdrawal_requests: u64,
//     pub receipt_token_to_process: u64,
//     pub receipt_token_being_processed: u64,
//     pub receipt_token_processed: u64,
//     pub sol_reserved: u64,
// }

// /// When operator ended processing the batch in progress
// #[event]
// pub struct FundEndBatchWithdrawalEvent {
//     pub receipt_token_mint: Pubkey,
//     pub batch_id: u64,
//     pub num_withdrawal_requests_processed: u64,
//     pub total_withdrawal_requests_processed: u64,
//     pub receipt_token_processed: u64,
//     pub total_receipt_token_processed: u64,
//     pub sol_reserved: u64,
//     pub total_sol_reserved: u64,
//     pub sol_remaining: u64,
// }
