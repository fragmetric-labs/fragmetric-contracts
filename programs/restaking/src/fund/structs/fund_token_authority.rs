use anchor_lang::prelude::*;

use crate::PDASignerSeeds;

#[account]
#[derive(InitSpace)]
pub struct FundTokenAuthority {
    pub data_version: u8,
    pub bump: u8,
    pub receipt_token_mint: Pubkey,
}

impl PDASignerSeeds<3> for FundTokenAuthority {
    const SEED: &'static [u8] = b"fund_token_authority_seed";

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
