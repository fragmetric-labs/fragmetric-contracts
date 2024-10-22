use anchor_lang::prelude::*;

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

    pub receipt_token_amount: u64,
    pub _reserved: [u8; 32],

    #[max_len(MAX_WITHDRAWAL_REQUESTS_SIZE)]
    pub withdrawal_requests: Vec<WithdrawalRequest>,
}

impl PDASeeds<3> for UserFundAccount {
    const SEED: &'static [u8] = b"user_fund";

    fn seeds(&self) -> [&[u8]; 3] {
        [
            Self::SEED,
            self.receipt_token_mint.as_ref(),
            self.user.as_ref(),
        ]
    }

    fn bump_ref(&self) -> &u8 {
        &self.bump
    }
}

impl UserFundAccount {
    pub const MAX_WITHDRAWAL_REQUESTS_SIZE: usize = MAX_WITHDRAWAL_REQUESTS_SIZE;

    pub fn initialize(&mut self, bump: u8, receipt_token_mint: Pubkey, user: Pubkey) {
        if self.data_version == 0 {
            self.data_version = 1;
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
            self.user = user;
        }
    }

    pub fn update_if_needed(&mut self, receipt_token_mint: Pubkey, user: Pubkey) {
        self.initialize(self.bump, receipt_token_mint, user);
    }

    pub fn placeholder(
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

    pub fn set_receipt_token_amount(&mut self, total_amount: u64) {
        self.receipt_token_amount = total_amount;
    }

    fn push_withdrawal_request(&mut self, request: WithdrawalRequest) -> Result<()> {
        require_gt!(
            Self::MAX_WITHDRAWAL_REQUESTS_SIZE,
            self.withdrawal_requests.len(),
            ErrorCode::FundExceededMaxWithdrawalRequestError
        );

        self.withdrawal_requests.push(request);

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
    pub fn create_withdrawal_request(
        &mut self,
        withdrawal_status: &mut WithdrawalStatus,
        receipt_token_amount: u64,
        current_time: i64,
    ) -> Result<(u64, u64)> {
        withdrawal_status.check_withdrawal_enabled()?;

        let request = WithdrawalRequest::new(
            withdrawal_status.pending_batch_withdrawal.batch_id,
            withdrawal_status.issue_new_request_id(),
            receipt_token_amount,
            current_time,
        );
        let batch_id = request.batch_id;
        let request_id = request.request_id;

        self.push_withdrawal_request(request)?;
        withdrawal_status
            .pending_batch_withdrawal
            .add_receipt_token_to_process(receipt_token_amount)?;

        Ok((batch_id, request_id))
    }

    pub fn cancel_withdrawal_request(
        &mut self,
        withdrawal_status: &mut WithdrawalStatus,
        request_id: u64,
    ) -> Result<WithdrawalRequest> {
        require_gt!(
            withdrawal_status.next_request_id,
            request_id,
            ErrorCode::FundWithdrawalRequestNotFoundError
        );

        let request = self.pop_withdrawal_request(request_id)?;
        withdrawal_status.check_batch_processing_not_started(request.batch_id)?;
        withdrawal_status
            .pending_batch_withdrawal
            .remove_receipt_token_to_process(request.receipt_token_amount)?;

        Ok(request)
    }

    pub fn pop_completed_withdrawal_request(
        &mut self,
        withdrawal_status: &mut WithdrawalStatus,
        request_id: u64,
    ) -> Result<WithdrawalRequest> {
        require_gt!(
            withdrawal_status.next_request_id,
            request_id,
            ErrorCode::FundWithdrawalRequestNotFoundError
        );

        withdrawal_status.check_withdrawal_enabled()?;
        let request = self.pop_withdrawal_request(request_id)?;
        withdrawal_status.check_batch_processing_completed(request.batch_id)?;

        Ok(request)
    }
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct WithdrawalRequest {
    pub batch_id: u64,
    pub request_id: u64,
    pub receipt_token_amount: u64,
    pub created_at: i64,
    pub _reserved: [u8; 16],
}

impl WithdrawalRequest {
    pub fn new(
        batch_id: u64,
        request_id: u64,
        receipt_token_amount: u64,
        current_time: i64,
    ) -> Self {
        Self {
            batch_id,
            request_id,
            receipt_token_amount,
            created_at: current_time,
            _reserved: [0; 16],
        }
    }
}
