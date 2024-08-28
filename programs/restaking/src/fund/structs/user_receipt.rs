use anchor_lang::prelude::*;

use crate::PDASignerSeeds;

#[account]
#[derive(InitSpace)]
pub struct UserReceipt {
    pub data_version: u8,
    pub bump: u8,
    pub user: Pubkey,
    pub receipt_token_mint: Pubkey,
    pub receipt_token_amount: u64,
    #[max_len(32)]
    pub withdrawal_requests: Vec<WithdrawalRequest>,
}

impl PDASignerSeeds<4> for UserReceipt {
    const SEED: &'static [u8] = b"user_receipt_seed_v2";

    fn signer_seeds(&self) -> [&[u8]; 4] {
        [
            Self::SEED,
            self.user.as_ref(),
            self.receipt_token_mint.as_ref(),
            self.bump_as_slice(),
        ]
    }

    fn bump_ref(&self) -> &u8 {
        &self.bump
    }
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct WithdrawalRequest {
    pub batch_id: u64,
    pub request_id: u64,
    pub receipt_token_amount: u64,
    pub created_at: i64,
}

impl WithdrawalRequest {
    pub fn new(batch_id: u64, request_id: u64, receipt_token_amount: u64) -> Result<Self> {
        Ok(Self {
            batch_id,
            request_id,
            receipt_token_amount,
            created_at: crate::utils::timestamp_now()?,
        })
    }
}
