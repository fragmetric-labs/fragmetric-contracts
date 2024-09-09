use anchor_lang::prelude::*;

use crate::modules::common::PDASignerSeeds;

const MAX_WITHDRAWAL_REQUESTS_SIZE: usize = 10;

#[account]
#[derive(InitSpace)]
pub struct UserFundAccount {
    data_version: u16,
    pub bump: u8,
    pub receipt_token_mint: Pubkey,
    pub user: Pubkey,

    pub receipt_token_amount: u64,
    pub _reserved: [u8; 32],

    #[max_len(MAX_WITHDRAWAL_REQUESTS_SIZE)]
    pub withdrawal_requests: Vec<WithdrawalRequest>,
}

impl PDASignerSeeds<4> for UserFundAccount {
    const SEED: &'static [u8] = b"user_fund";

    fn signer_seeds(&self) -> [&[u8]; 4] {
        [
            Self::SEED,
            self.receipt_token_mint.as_ref(),
            self.user.as_ref(),
            self.bump_as_slice(),
        ]
    }

    fn bump_ref(&self) -> &u8 {
        &self.bump
    }
}

impl UserFundAccount {
    pub const MAX_WITHDRAWAL_REQUESTS_SIZE: usize = MAX_WITHDRAWAL_REQUESTS_SIZE;

    pub fn initialize_if_needed(&mut self, bump: u8, receipt_token_mint: Pubkey, user: Pubkey) {
        if self.data_version == 0 {
            self.data_version = 1;
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
            self.user = user;
        }
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
    pub fn new(batch_id: u64, request_id: u64, receipt_token_amount: u64) -> Result<Self> {
        Ok(Self {
            batch_id,
            request_id,
            receipt_token_amount,
            created_at: crate::utils::timestamp_now()?,
            _reserved: [0; 16],
        })
    }
}
