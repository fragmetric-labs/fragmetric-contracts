use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::errors::ErrorCode;
use crate::utils::PDASeeds;

use super::*;

const MAX_WITHDRAWAL_REQUESTS_SIZE: usize = 10;

#[account]
#[derive(InitSpace)]
pub struct UserFundAccount {
    data_version: u16,
    bump: u8,
    pub receipt_token_mint: Pubkey,
    pub user: Pubkey,

    pub(super) receipt_token_amount: u64,
    _reserved: [u8; 32],

    #[max_len(MAX_WITHDRAWAL_REQUESTS_SIZE)]
    withdrawal_requests: Vec<WithdrawalRequest>,
}

impl PDASeeds<3> for UserFundAccount {
    const SEED: &'static [u8] = b"user_fund";

    fn get_seed_phrase(&self) -> [&[u8]; 3] {
        [
            Self::SEED,
            self.receipt_token_mint.as_ref(),
            self.user.as_ref(),
        ]
    }

    fn get_bump_ref(&self) -> &u8 {
        &self.bump
    }
}

impl UserFundAccount {
    const MAX_WITHDRAWAL_REQUESTS_SIZE: usize = MAX_WITHDRAWAL_REQUESTS_SIZE;

    fn migrate(
        &mut self,
        bump: u8,
        receipt_token_mint: Pubkey,
        receipt_token_amount: u64,
        user: Pubkey,
    ) {
        if self.data_version == 0 {
            self.data_version = 1;
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
            self.receipt_token_amount = receipt_token_amount;
            self.user = user;
        }
    }

    #[inline(always)]
    pub(super) fn initialize(
        &mut self,
        bump: u8,
        receipt_token_mint: &InterfaceAccount<Mint>,
        user_receipt_token_account: &InterfaceAccount<TokenAccount>,
    ) {
        self.migrate(
            bump,
            receipt_token_mint.key(),
            user_receipt_token_account.amount,
            user_receipt_token_account.owner,
        );
    }

    #[inline(always)]
    pub(super) fn update_if_needed(
        &mut self,
        receipt_token_mint: &InterfaceAccount<Mint>,
        user_receipt_token_account: &InterfaceAccount<TokenAccount>,
    ) {
        self.initialize(self.bump, receipt_token_mint, user_receipt_token_account);
    }

    // create a placeholder to emit event for non existing user account
    pub(super) fn placeholder(
        user: Pubkey,
        receipt_token_mint: Pubkey,
        receipt_token_amount: u64,
    ) -> Self {
        Self {
            data_version: 0,
            bump: 0,
            receipt_token_mint,
            user,
            receipt_token_amount,
            _reserved: [0; 32],
            withdrawal_requests: Default::default(),
        }
    }

    pub(super) fn reload_receipt_token_amount(
        &mut self,
        user_receipt_token_account: &mut InterfaceAccount<TokenAccount>,
    ) -> Result<()> {
        #[cfg(debug_assertions)]
        require_keys_eq!(self.user, user_receipt_token_account.owner);

        #[cfg(debug_assertions)]
        require_keys_eq!(self.receipt_token_mint, user_receipt_token_account.mint);

        user_receipt_token_account.reload()?;
        self.receipt_token_amount = user_receipt_token_account.amount;

        Ok(())
    }

    fn pop_withdrawal_request(&mut self, request_id: u64) -> Result<WithdrawalRequest> {
        let index = self
            .withdrawal_requests
            .binary_search_by_key(&request_id, |req| req.request_id)
            .map_err(|_| error!(ErrorCode::FundWithdrawalRequestNotFoundError))?;
        Ok(self.withdrawal_requests.remove(index))
    }

    /// Returns (batch_id, request_id)
    pub(super) fn create_withdrawal_request(
        &mut self,
        withdrawal_status: &mut WithdrawalState,
        receipt_token_amount: u64,
        current_timestamp: i64,
    ) -> Result<(u64, u64)> {
        require_gt!(
            Self::MAX_WITHDRAWAL_REQUESTS_SIZE,
            self.withdrawal_requests.len(),
            ErrorCode::FundExceededMaxWithdrawalRequestError
        );

        let request = withdrawal_status
            .issue_new_withdrawal_request(receipt_token_amount, current_timestamp)?;
        let batch_id = request.batch_id;
        let request_id = request.request_id;

        self.withdrawal_requests.push(request);

        Ok((batch_id, request_id))
    }

    /// Returns receipt_token_amount
    pub(super) fn cancel_withdrawal_request(
        &mut self,
        withdrawal_status: &mut WithdrawalState,
        request_id: u64,
    ) -> Result<u64> {
        let request = self.pop_withdrawal_request(request_id)?;
        withdrawal_status.remove_withdrawal_request_from_batch(request)
    }

    /// Returns (sol_user_amount, sol_fee_amount, receipt_token_burn_amount)
    pub(super) fn claim_withdrawal_request(
        &mut self,
        withdrawal_status: &mut WithdrawalState,
        request_id: u64,
    ) -> Result<(u64, u64, u64)> {
        let request = self.pop_withdrawal_request(request_id)?;
        withdrawal_status.claim_withdrawal_request(request)
    }
}
