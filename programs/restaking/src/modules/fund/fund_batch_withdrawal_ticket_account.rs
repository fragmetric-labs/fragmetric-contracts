use anchor_lang::prelude::*;

use crate::errors::ErrorCode;

use super::*;

#[account]
#[derive(InitSpace)]
pub struct FundBatchWithdrawalTicketAccount {
    data_version: u16,
    bump: u8,
    pub receipt_token_mint: Pubkey,
    pub batch_id: u64,
    num_requests: u64,
    receipt_token_amount: u64,
    claimed_receipt_token_amount: u64,
    pub(super) sol_user_amount: u64,
    claimed_sol_user_amount: u64,
    /// SOL withdrawal fee is already paid to treasury account.
    /// This field is just for event.
    pub(super) sol_fee_amount: u64,
    processed_at: i64,
    _reserved: [u8; 32],
}

impl FundBatchWithdrawalTicketAccount {
    pub const SEED: &'static [u8] = b"batch_withdrawal_ticket";

    pub fn get_bump(&self) -> u8 {
        self.bump
    }

    #[inline(always)]
    fn get_seed_phrase(receipt_token_mint: &Pubkey, batch_id: u64) -> [Vec<u8>; 3] {
        [
            Self::SEED.to_vec(),
            receipt_token_mint.as_ref().to_vec(),
            batch_id.to_le_bytes().to_vec(),
        ]
    }

    /// usage:
    /// ```rs
    /// let seeds: Vec<Vec<u8>> = get_seeds();
    /// let seeds_ref: &[&[u8]] = seeds.iter().map(Vec::as_slice).collect::<Vec<_>>().as_slice();
    /// // ...
    /// ctx.with_signer_seeds(&[seeds_ref])
    /// ```
    pub(super) fn get_seeds(receipt_token_mint: &Pubkey, batch_id: u64) -> Vec<Vec<u8>> {
        let seed_phrase = Self::get_seed_phrase(receipt_token_mint, batch_id);
        let bump = Pubkey::find_program_address(
            &std::array::from_fn::<_, 4, _>(|i| seed_phrase[i].as_slice()),
            &crate::ID,
        )
        .1;

        let mut seeds = Vec::with_capacity(4);
        seeds.extend(seed_phrase);
        seeds.push(vec![bump]);
        seeds
    }

    pub(super) fn find_account_address(receipt_token_mint: &Pubkey, batch_id: u64) -> (Pubkey, u8) {
        let seed_phrase = Self::get_seed_phrase(receipt_token_mint, batch_id);
        Pubkey::find_program_address(
            &std::array::from_fn::<_, 4, _>(|i| seed_phrase[i].as_slice()),
            &crate::ID,
        )
    }

    fn migrate(&mut self, bump: u8, receipt_token_mint: Pubkey, batch_id: u64) {
        if self.data_version == 0 {
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
            self.batch_id = batch_id;
            self._reserved = Default::default();
            self.data_version = 1;
        }
    }

    #[inline(always)]
    pub(super) fn initialize(&mut self, bump: u8, receipt_token_mint: Pubkey, batch_id: u64) {
        self.migrate(bump, receipt_token_mint, batch_id)
    }

    #[inline(always)]
    pub(super) fn update_if_needed(&mut self, receipt_token_mint: Pubkey, batch_id: u64) {
        self.migrate(self.bump, receipt_token_mint, batch_id)
    }

    pub(super) fn set_withdrawal_amount(
        &mut self,
        num_requests: u64,
        receipt_token_amount: u64,
        sol_user_amount: u64,
        sol_fee_amount: u64,
        processed_at: i64,
    ) {
        self.num_requests = num_requests;
        self.receipt_token_amount = receipt_token_amount;
        self.claimed_receipt_token_amount = 0;
        self.sol_user_amount = sol_user_amount;
        self.claimed_sol_user_amount = 0;
        self.sol_fee_amount = sol_fee_amount;
        self.processed_at = processed_at;
    }

    pub(super) fn is_stale(&self) -> bool {
        self.claimed_receipt_token_amount == self.receipt_token_amount
    }

    /// Returns (sol_user_amount, sol_fee_amount, receipt_token_withdraw_amount)
    pub(super) fn claim_withdrawal_request(
        &mut self,
        request: WithdrawalRequest,
    ) -> Result<(u64, u64, u64)> {
        require_eq!(
            self.batch_id,
            request.batch_id,
            ErrorCode::FundWrongWithdrawalBatchError,
        );

        let sol_user_amount = crate::utils::get_proportional_amount(
            request.receipt_token_amount,
            self.sol_user_amount,
            self.receipt_token_amount,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        // only informational
        let sol_fee_amount = crate::utils::get_proportional_amount(
            request.receipt_token_amount,
            self.sol_fee_amount,
            self.receipt_token_amount,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        self.claimed_receipt_token_amount += request.receipt_token_amount;
        self.claimed_sol_user_amount += sol_user_amount;

        Ok((
            sol_user_amount,
            sol_fee_amount,
            request.receipt_token_amount,
        ))
    }
}
