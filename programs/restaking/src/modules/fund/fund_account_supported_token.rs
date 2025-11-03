use anchor_lang::prelude::*;
use bytemuck::Zeroable;

use crate::errors::ErrorCode;
use crate::modules::pricing::{TokenPricingSource, TokenPricingSourcePod};

use super::AssetState;

#[zero_copy]
pub(super) struct SupportedToken {
    pub mint: Pubkey,
    pub program: Pubkey,
    pub decimals: u8,
    _padding: [u8; 7],

    pub pricing_source: TokenPricingSourcePod,

    /// informative
    pub one_token_as_sol: u64,

    /// token deposit & withdrawal
    pub token: AssetState,
    _padding2: [u8; 8],

    /// configuration: used for staking allocation strategy.
    pub sol_allocation_weight: u64,
    pub sol_allocation_capacity_amount: u64,

    // third parties state tracking
    pub pending_unstaking_amount_as_sol: u64,

    /// informative
    pub one_token_as_receipt_token: u64,

    _reserved: [u8; 48],
}

impl SupportedToken {
    pub fn initialize(
        &mut self,
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        pricing_source: TokenPricingSource,
        operation_reserved_amount: u64,
    ) -> Result<()> {
        match pricing_source {
            TokenPricingSource::SPLStakePool { .. }
            | TokenPricingSource::MarinadeStakePool { .. }
            | TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. }
            | TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. }
            | TokenPricingSource::OrcaDEXLiquidityPool { .. }
            | TokenPricingSource::PeggedToken { .. } => {}
            // otherwise fails
            TokenPricingSource::JitoRestakingVault { .. }
            | TokenPricingSource::SolvBTCVault { .. }
            | TokenPricingSource::VirtualVault { .. }
            | TokenPricingSource::FragmetricNormalizedTokenPool { .. }
            | TokenPricingSource::FragmetricRestakingFund { .. } => {
                err!(ErrorCode::FundNotSupportedTokenError)?
            }
            #[cfg(all(test, not(feature = "idl-build")))]
            TokenPricingSource::Mock { .. } => err!(ErrorCode::FundNotSupportedTokenError)?,
        }

        *self = Zeroable::zeroed();

        self.mint = mint;
        self.program = program;
        self.decimals = decimals;
        pricing_source.serialize_as_pod(&mut self.pricing_source);

        self.token
            .initialize(Some((mint, program)), operation_reserved_amount);
        Ok(())
    }

    pub fn set_sol_allocation_strategy(
        &mut self,
        weight: u64,
        sol_capacity_amount: u64,
    ) -> Result<()> {
        self.sol_allocation_weight = weight;
        self.sol_allocation_capacity_amount = sol_capacity_amount;

        Ok(())
    }
}
