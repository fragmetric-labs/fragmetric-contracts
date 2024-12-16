use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::errors::ErrorCode;
use crate::modules::pricing::{TokenPricingSource, TokenPricingSourcePod};

use super::AssetFlowState;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod, Debug)]
#[repr(C)]
pub(super) struct SupportedToken {
    pub mint: Pubkey,
    pub program: Pubkey,
    pub decimals: u8,
    _padding: [u8; 7],

    pub pricing_source: TokenPricingSourcePod,

    /// informative
    pub one_token_as_sol: u64,

    /// token deposit & withdrawal
    pub token_flow: AssetFlowState,

    /// asset
    pub operation_reserved_amount: u64,

    /// asset: the token amount being unstaked
    pub operation_receivable_amount: u64,

    /// configuration: the amount requested to be unstaked as soon as possible regardless of current state, this value should be decreased by each unstaking requested amount.
    pub rebalancing_amount: u64,

    /// configuration: used for staking allocation strategy.
    pub sol_allocation_weight: u64,
    pub sol_allocation_capacity_amount: u64,

    _reserved: [u8; 64],
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
            | TokenPricingSource::MarinadeStakePool { .. } => {}
            _ => {
                err!(ErrorCode::FundNotSupportedTokenError)?;
            }
        }

        self.mint = mint;
        self.program = program;
        self.decimals = decimals;
        self.operation_reserved_amount = operation_reserved_amount;
        pricing_source.serialize_as_pod(&mut self.pricing_source);

        self.token_flow.token_mint = mint;
        Ok(())
    }

    pub(super) fn set_sol_allocation_strategy(
        &mut self,
        weight: u64,
        sol_capacity_amount: u64,
    ) -> Result<()> {
        self.sol_allocation_weight = weight;
        self.sol_allocation_capacity_amount = sol_capacity_amount;

        Ok(())
    }

    pub(super) fn set_rebalancing_strategy(&mut self, token_amount: u64) -> Result<()> {
        require_gte!(
            token_amount,
            self.operation_reserved_amount,
            ErrorCode::FundInvalidUpdateError
        );

        self.rebalancing_amount = token_amount;

        Ok(())
    }

    pub(super) fn deposit_token(&mut self, amount: u64) -> Result<()> {
        let new_accumulated_deposit_amount = self
            .token_flow
            .accumulated_deposit_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        require_gte!(
            self.token_flow.accumulated_deposit_capacity_amount,
            new_accumulated_deposit_amount,
            ErrorCode::FundExceededTokenCapacityAmountError
        );

        self.token_flow.accumulated_deposit_amount = new_accumulated_deposit_amount;
        self.operation_reserved_amount = self
            .operation_reserved_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }
}
