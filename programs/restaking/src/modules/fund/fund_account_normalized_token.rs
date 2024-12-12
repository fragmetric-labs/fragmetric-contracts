use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::modules::pricing::{TokenPricingSource, TokenPricingSourcePod};
use crate::modules::reward::RewardType::Token;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod, Debug)]
#[repr(C)]
pub(super) struct NormalizedToken {
    pub mint: Pubkey,
    pub program: Pubkey,
    pub decimals: u8,
    pub enabled: u8,
    _padding: [u8; 6],
    pub pricing_source: TokenPricingSourcePod,
    pub one_token_as_sol: u64,
    pub operation_reserved_amount: u64,
    _reserved: [u8; 64],
}

impl NormalizedToken {
    pub fn initialize(
        &mut self,
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        pool: Pubkey,
        operation_reserved_amount: u64,
    ) -> Result<()> {
        require_eq!(self.enabled, 0);

        self.enabled = 1;
        self.mint = mint;
        self.program = program;
        self.decimals = decimals;
        TokenPricingSource::FragmetricNormalizedTokenPool { address: pool }
            .serialize_as_pod(&mut self.pricing_source);
        self.operation_reserved_amount = operation_reserved_amount;

        Ok(())
    }
}
