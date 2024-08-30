mod info;
mod user_canceled_withdrawal_request_from_fund;
mod user_deposited_sol_to_fund;
mod user_deposited_token_to_fund;
mod user_requested_withdrawal_from_fund;
mod user_updated_fund_price;
mod user_withdrew_sol_from_fund;

pub use info::*;
pub use user_canceled_withdrawal_request_from_fund::*;
pub use user_deposited_sol_to_fund::*;
pub use user_deposited_token_to_fund::*;
pub use user_requested_withdrawal_from_fund::*;
pub use user_updated_fund_price::*;
pub use user_withdrew_sol_from_fund::*;

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
