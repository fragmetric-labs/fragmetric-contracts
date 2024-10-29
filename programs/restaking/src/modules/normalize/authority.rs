use anchor_lang::prelude::*;

use crate::utils::PDASeeds;

#[account]
#[derive(InitSpace)]
pub struct NormalizedTokenAuthority {
    data_version: u16,
    bump: u8,
    pub receipt_token_mint: Pubkey,
    pub normalized_token_mint: Pubkey,
}

impl PDASeeds<2> for NormalizedTokenAuthority {
    const SEED: &'static [u8] = b"normalized_token_authority";

    fn get_seeds(&self) -> [&[u8]; 2] {
        [
            self.receipt_token_mint.as_ref(),
            self.normalized_token_mint.as_ref(),
        ]
    }

    fn get_bump_ref(&self) -> &u8 {
        &self.bump
    }
}

impl NormalizedTokenAuthority {
    pub const NORMALIZED_TOKEN_ACCOUNT_SEED: &'static [u8] = b"normalized_token";
    pub const SUPPORTED_TOKEN_LOCK_ACCOUNT_SEED: &'static [u8] = b"supported_token_lock";

    pub(super) fn initialize(
        &mut self,
        bump: u8,
        receipt_token_mint: Pubkey,
        normalized_token_mint: Pubkey,
    ) {
        self.bump = bump;
        self.receipt_token_mint = receipt_token_mint;
        self.normalized_token_mint = normalized_token_mint;
    }
}
