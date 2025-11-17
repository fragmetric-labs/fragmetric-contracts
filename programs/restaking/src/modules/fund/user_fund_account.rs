use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::errors::ErrorCode;
use crate::utils::PDASeeds;

#[constant]
/// ## Version History
/// * v1: Initial Version (567 ~= 0.55KB)
pub const USER_FUND_ACCOUNT_CURRENT_VERSION: u16 = 1;

pub const USER_FUND_ACCOUNT_MAX_WITHDRAWAL_REQUESTS_SIZE: usize = 4;

#[account]
#[derive(InitSpace)]
pub struct UserFundAccount {
    data_version: u16,
    bump: u8,
    pub receipt_token_mint: Pubkey,
    pub user: Pubkey,

    pub(super) receipt_token_amount: u64,
    _reserved: [u8; 32],

    #[max_len(USER_FUND_ACCOUNT_MAX_WITHDRAWAL_REQUESTS_SIZE)]
    withdrawal_requests: Vec<WithdrawalRequest>,
}

impl PDASeeds<4> for UserFundAccount {
    const SEED: &'static [u8] = b"user_fund";

    fn get_bump(&self) -> u8 {
        self.bump
    }

    fn get_seeds(&self) -> [&[u8]; 4] {
        [
            Self::SEED,
            self.receipt_token_mint.as_ref(),
            self.user.as_ref(),
            core::slice::from_ref(&self.bump),
        ]
    }
}

impl UserFundAccount {
    fn migrate(
        &mut self,
        bump: u8,
        receipt_token_mint: Pubkey,
        receipt_token_amount: u64,
        user: Pubkey,
    ) -> Result<bool> {
        let old_data_version = self.data_version;

        if self.data_version == 0 {
            self.data_version = 1;
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
            self.receipt_token_amount = receipt_token_amount;
            self.user = user;
        }

        require_eq!(self.data_version, USER_FUND_ACCOUNT_CURRENT_VERSION);

        Ok(old_data_version < self.data_version)
    }

    #[inline(always)]
    pub(super) fn initialize(
        &mut self,
        user_fund_account_bump: u8,
        receipt_token_mint: &InterfaceAccount<Mint>,
        user_receipt_token_account: &InterfaceAccount<TokenAccount>,
    ) -> Result<bool> {
        self.migrate(
            user_fund_account_bump,
            receipt_token_mint.key(),
            user_receipt_token_account.amount,
            user_receipt_token_account.owner,
        )
    }

    #[inline(always)]
    pub(super) fn update_if_needed(
        &mut self,
        receipt_token_mint: &InterfaceAccount<Mint>,
        user_receipt_token_account: &InterfaceAccount<TokenAccount>,
    ) -> Result<bool> {
        self.initialize(self.bump, receipt_token_mint, user_receipt_token_account)
    }

    #[inline(always)]
    pub(super) fn is_initializing(&self) -> bool {
        self.data_version == 0
    }

    #[inline(always)]
    pub fn is_latest_version(&self) -> bool {
        self.data_version == USER_FUND_ACCOUNT_CURRENT_VERSION
    }

    pub(super) fn reload_receipt_token_amount(
        &mut self,
        user_receipt_token_account: &mut InterfaceAccount<TokenAccount>,
    ) -> Result<()> {
        require_keys_eq!(self.user, user_receipt_token_account.owner);

        require_keys_eq!(self.receipt_token_mint, user_receipt_token_account.mint);

        user_receipt_token_account.reload()?;
        self.receipt_token_amount = user_receipt_token_account.amount;

        Ok(())
    }

    pub(super) fn push_withdrawal_request(&mut self, request: WithdrawalRequest) -> Result<()> {
        require_gt!(
            USER_FUND_ACCOUNT_MAX_WITHDRAWAL_REQUESTS_SIZE,
            self.withdrawal_requests.len(),
            ErrorCode::FundExceededMaxWithdrawalRequestError
        );

        self.withdrawal_requests.push(request);
        Ok(())
    }

    pub(super) fn pop_withdrawal_request(
        &mut self,
        request_id: u64,
        supported_token_mint: Option<Pubkey>,
    ) -> Result<WithdrawalRequest> {
        let index = self
            .withdrawal_requests
            .iter()
            .enumerate()
            .find_map(|(index, request)| {
                (request.request_id == request_id
                    && request.supported_token_mint == supported_token_mint)
                    .then_some(index)
            })
            .ok_or_else(|| error!(ErrorCode::FundWithdrawalRequestNotFoundError))?;
        Ok(self.withdrawal_requests.remove(index))
    }

    pub(super) fn is_withdrawal_requests_empty(&self) -> bool {
        self.withdrawal_requests.len() == 0
    }
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct WithdrawalRequest {
    pub batch_id: u64,
    pub request_id: u64,
    pub receipt_token_amount: u64,
    created_at: i64,
    pub supported_token_mint: Option<Pubkey>,
    pub supported_token_program: Option<Pubkey>,
    _reserved: [u8; 14],
}

impl WithdrawalRequest {
    pub fn new(
        batch_id: u64,
        request_id: u64,
        receipt_token_amount: u64,
        supported_token_mint_and_program: Option<(Pubkey, Pubkey)>,
        current_timestamp: i64,
    ) -> Self {
        Self {
            batch_id,
            request_id,
            receipt_token_amount,
            supported_token_mint: supported_token_mint_and_program.map(|(mint, _)| mint),
            supported_token_program: supported_token_mint_and_program.map(|(_, program)| program),
            created_at: current_timestamp,
            _reserved: [0; 14],
        }
    }
}
