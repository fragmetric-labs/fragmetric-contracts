use anchor_lang::prelude::*;
use bytemuck::{Zeroable, Pod};

use crate::modules::pricing::{TokenPricingSource, TokenPricingSourcePod};
use crate::modules::reward::RewardType::Token;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod, Debug)]
#[repr(C, align(16))]
pub(super) struct NormalizedToken {
    pub mint: Pubkey,
    pub program: Pubkey,
    pub decimals: u8,
    _padding: [u8; 15],
    pub pricing_source: TokenPricingSourcePod,
    pub one_token_as_sol: u64,
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
            _padding: [0; 15],
            pricing_source: TokenPricingSource::FragmetricNormalizedTokenPool { address: pool }.into(),
            one_token_as_sol: 0,
            operation_reserved_amount,
            _reserved: [0; 64],
        }
    }
}
