use anchor_lang::prelude::*;

use crate::utils::PDASeeds;

#[account]
#[derive(InitSpace)]
pub struct NormalizedTokenAuthority {
    data_version: u16,
    bump: u8,
    pub normalized_token_mint: Pubkey,
}

impl PDASeeds<1> for NormalizedTokenAuthority {
    const SEED: &'static [u8] = b"normalized_token_authority";

    fn get_seeds(&self) -> [&[u8]; 1] {
        [self.normalized_token_mint.as_ref()]
    }

    fn get_bump_ref(&self) -> &u8 {
        &self.bump
    }
}

impl NormalizedTokenAuthority {
    pub const NORMALIZED_TOKEN_ACCOUNT_SEED: &'static [u8] = b"normalized_token";
    pub const SUPPORTED_TOKEN_LOCK_ACCOUNT_SEED: &'static [u8] = b"supported_token_lock";

    pub(super) fn initialize(&mut self, bump: u8, normalized_token_mint: Pubkey) {
        self.bump = bump;
        self.normalized_token_mint = normalized_token_mint;
    }
}
