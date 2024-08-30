use anchor_lang::prelude::*;

use crate::PDASignerSeeds;

#[account]
#[derive(InitSpace)]
pub struct ReceiptTokenLockAuthority {
    pub data_version: u8,
    pub bump: u8,
    pub receipt_token_mint: Pubkey,
}

impl PDASignerSeeds<3> for ReceiptTokenLockAuthority {
    const SEED: &'static [u8] = b"receipt_token_lock_authority";

    fn signer_seeds(&self) -> [&[u8]; 3] {
        [
            Self::SEED,
            self.receipt_token_mint.as_ref(),
            self.bump_as_slice(),
        ]
    }

    fn bump_ref(&self) -> &u8 {
        &self.bump
    }
}

#[account]
#[derive(InitSpace)]
pub struct SupportedTokenAuthority {
    pub data_version: u8,
    pub bump: u8,
    pub receipt_token_mint: Pubkey,
    pub supported_token_mint: Pubkey,
}

impl PDASignerSeeds<4> for SupportedTokenAuthority {
    const SEED: &'static [u8] = b"supported_token_authority";

    fn signer_seeds(&self) -> [&[u8]; 4] {
        [
            Self::SEED,
            self.receipt_token_mint.as_ref(),
            self.supported_token_mint.as_ref(),
            self.bump_as_slice(),
        ]
    }

    fn bump_ref(&self) -> &u8 {
        &self.bump
    }
}

#[account]
#[derive(InitSpace)]
pub struct ReceiptTokenMintAuthority {
    pub data_version: u8,
    pub bump: u8,
    pub receipt_token_mint: Pubkey,
}

impl PDASignerSeeds<3> for ReceiptTokenMintAuthority {
    const SEED: &'static [u8] = b"receipt_token_mint_authority";

    fn signer_seeds(&self) -> [&[u8]; 3] {
        [
            Self::SEED,
            self.receipt_token_mint.as_ref(),
            self.bump_as_slice(),
        ]
    }

    fn bump_ref(&self) -> &u8 {
        &self.bump
    }
}
