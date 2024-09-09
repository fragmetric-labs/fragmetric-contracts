use anchor_lang::prelude::*;

use crate::modules::common::PDASignerSeeds;

#[account]
#[derive(InitSpace)]
pub struct ReceiptTokenLockAuthority {
    data_version: u16,
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

impl ReceiptTokenLockAuthority {
    pub const TOKEN_ACCOUNT_SEED: &'static [u8] = b"receipt_token_lock";

    pub fn initialize_if_needed(&mut self, bump: u8, receipt_token_mint: Pubkey) {
        if self.data_version == 0 {
            self.data_version = 1;
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
        }
    }
}

#[account]
#[derive(InitSpace)]
pub struct SupportedTokenAuthority {
    data_version: u16,
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

impl SupportedTokenAuthority {
    pub const TOKEN_ACCOUNT_SEED: &'static [u8] = b"supported_token";

    pub fn initialize_if_needed(
        &mut self,
        bump: u8,
        receipt_token_mint: Pubkey,
        supported_token_mint: Pubkey,
    ) {
        if self.data_version == 0 {
            self.data_version = 1;
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
            self.supported_token_mint = supported_token_mint;
        }
    }
}

#[account]
#[derive(InitSpace)]
pub struct ReceiptTokenMintAuthority {
    data_version: u16,
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

impl ReceiptTokenMintAuthority {
    pub fn initialize_if_needed(&mut self, bump: u8, receipt_token_mint: Pubkey) {
        if self.data_version == 0 {
            self.data_version = 1;
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
        }
    }
}
