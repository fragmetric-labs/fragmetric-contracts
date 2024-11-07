use anchor_lang::prelude::*;

use crate::utils::PDASeeds;

#[account]
#[derive(InitSpace)]
pub struct ReceiptTokenLockAuthority {
    data_version: u16,
    bump: u8,
    pub receipt_token_mint: Pubkey,
}

impl PDASeeds<2> for ReceiptTokenLockAuthority {
    const SEED: &'static [u8] = b"receipt_token_lock_authority";

    fn get_seeds(&self) -> [&[u8]; 2] {
        [Self::SEED, self.receipt_token_mint.as_ref()]
    }

    fn get_bump_ref(&self) -> &u8 {
        &self.bump
    }
}

impl ReceiptTokenLockAuthority {
    pub const TOKEN_ACCOUNT_SEED: &'static [u8] = b"receipt_token_lock";
}

#[account]
#[derive(InitSpace)]
pub struct SupportedTokenAuthority {
    data_version: u16,
    bump: u8,
    pub receipt_token_mint: Pubkey,
    pub supported_token_mint: Pubkey,
}

impl PDASeeds<3> for SupportedTokenAuthority {
    const SEED: &'static [u8] = b"supported_token_authority";

    fn get_seeds(&self) -> [&[u8]; 3] {
        [
            Self::SEED,
            self.receipt_token_mint.as_ref(),
            self.supported_token_mint.as_ref(),
        ]
    }

    fn get_bump_ref(&self) -> &u8 {
        &self.bump
    }
}

impl SupportedTokenAuthority {
    pub const TOKEN_ACCOUNT_SEED: &'static [u8] = b"supported_token";
}

#[account]
#[derive(InitSpace)]
pub struct ReceiptTokenMintAuthority {
    data_version: u16,
    bump: u8,
    pub receipt_token_mint: Pubkey,
}

impl PDASeeds<2> for ReceiptTokenMintAuthority {
    const SEED: &'static [u8] = b"receipt_token_mint_authority";

    fn get_seeds(&self) -> [&[u8]; 2] {
        [Self::SEED, self.receipt_token_mint.as_ref()]
    }

    fn get_bump_ref(&self) -> &u8 {
        &self.bump
    }
}
