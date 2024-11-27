use anchor_lang::prelude::*;

use crate::modules::pricing::TokenPricingSource;
use crate::modules::reward::RewardType::Token;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct NormalizedToken {
    pub mint: Pubkey,
    pub program: Pubkey,
    pub decimals: u8,
    pub one_token_as_sol: u64,
    pub pricing_source: TokenPricingSource,
    pub operation_reserved_amount: u64,
    _reserved: [u8; 64],
}

impl NormalizedToken {
    pub fn new(
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        pool: Pubkey,
        operation_reserved_amount: u64,
    ) -> Self {
        Self {
            mint,
            program,
            decimals,
            one_token_as_sol: 0,
            pricing_source: TokenPricingSource::FragmetricNormalizedTokenPool { address: pool },
            operation_reserved_amount,
            _reserved: [0; 64],
        }
    }
}
