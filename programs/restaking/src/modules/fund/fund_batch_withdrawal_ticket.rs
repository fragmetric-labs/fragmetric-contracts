// use anchor_lang::prelude::*;

// use crate::utils::PDASeeds;

// use super::WithdrawalBatch;

// #[account]
// #[derive(InitSpace)]
// pub struct FundBatchWithdrawalTicket {
//     data_version: u16,
//     bump: u8,
//     pub receipt_token_mint: Pubkey,
//     pub batch_id: u64,
//     num_requests: u64,
//     claimed_requests: u64,
//     receipt_token_amount: u64,
//     sol_withdrawal_amount: u64,
//     sol_fee_amount: u64,
//     processed_at: i64,
//     _reserved: [u8; 32],
// }

// impl PDASeeds<3> for FundBatchWithdrawalTicket {
//     const SEED: &'static [u8] = b"fund_batch_withdrawal_ticket";

//     fn get_seed_phrase(&self) -> [&[u8]; 3] {
//         // SAFETY: solana runtime is little endian and does not care alignment.
//         let batch_id = unsafe { &*(self.batch_id as *const u64 as *const [u8; 8]) }.as_slice();
//         [Self::SEED, self.receipt_token_mint.as_ref(), batch_id]
//     }

//     fn get_bump_ref(&self) -> &u8 {
//         &self.bump
//     }
// }

// impl FundBatchWithdrawalTicket {
//     fn migrate(
//         &mut self,
//         bump: u8,
//         receipt_token_mint: Pubkey,
//         batch_id: u64,
//         num_requests: u64,
//         receipt_token_amount: u64,
//         sol_amount: u64,
//         processed_at: i64,
//     ) {
//         if self.data_version == 0 {
//             self.bump = bump;
//             self.receipt_token_mint = receipt_token_mint;
//             self.batch_id = batch_id;
//             self.num_requests = num_requests;
//             self.claimed_requests = 0;
//             self.receipt_token_amount = receipt_token_amount;
//             self.sol_withdrawal_amount = sol_amount;
//             self.processed_at = processed_at;
//             self._reserved = Default::default();
//             self.data_version = 1;
//         }
//     }

//     #[inline(always)]
//     pub(super) fn initialize(
//         &mut self,
//         bump: u8,
//         receipt_token_mint: Pubkey,
//         batch: WithdrawalBatch,
//     ) {
//         self.migrate(
//             bump,
//             receipt_token_mint,
//             batch.batch_id,
//             batch.num_requests,
//             batch.receipt_token_amount,
//             batch.sol_reserved,
//             batch.processed_at.unwrap(),
//         );
//     }

//     #[inline(always)]
//     pub(super) fn update_if_needed(&mut self, receipt_token_mint: Pubkey, batch_id: u64) {
//         self.migrate(
//             self.bump,
//             receipt_token_mint,
//             batch_id,
//             self.num_requests,
//             self.receipt_token_amount,
//             self.sol_withdrawal_amount,
//             self.processed_at,
//         );
//     }

//     pub(super) fn find_account_address(&self) -> Result<Pubkey> {
//         Ok(
//             Pubkey::create_program_address(&self.get_seeds(), &crate::ID)
//                 .map_err(|_| ProgramError::InvalidSeeds)?,
//         )
//     }

//     pub(super) fn is_stale(&self) -> bool {
//         self.num_requests == self.claimed_requests
//     }
// }
