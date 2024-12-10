use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::errors::ErrorCode;
use crate::modules::pricing::{TokenPricingSource, TokenPricingSourcePod};

// TODO v0.3/operation: visibility
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod, Debug)]
#[repr(C)]
pub struct SupportedToken {
    pub mint: Pubkey,
    pub program: Pubkey,
    pub decimals: u8,
    _padding: [u8; 15],

    pub accumulated_deposit_capacity_amount: u64,
    pub accumulated_deposit_amount: u64,
    pub operation_reserved_amount: u64,
    pub one_token_as_sol: u64,
    pub pricing_source: TokenPricingSourcePod,

    /// the token amount being unstaked
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
        // TODO: operation_reserved_amount: u64,
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
        self.pricing_source = pricing_source.into();

        Ok(())
    }

    // TODO v0.3/operation: visibility
    pub(in crate::modules) fn get_operation_reserved_amount(&self) -> u64 {
        self.operation_reserved_amount
    }

    // TODO v0.3/operation: visibility
    pub(in crate::modules) fn set_operation_reserved_amount(&mut self, amount: u64) {
        self.operation_reserved_amount = amount;
    }

    // TODO v0.3/operation: visibility
    pub(in crate::modules) fn get_operating_amount(&self) -> u64 {
        self.operation_receivable_amount
    }

    // TODO v0.3/operation: visibility
    pub(in crate::modules) fn set_operating_amount(&mut self, amount: u64) {
        self.operation_receivable_amount = amount;
    }

    pub(super) fn set_accumulated_deposit_amount(&mut self, token_amount: u64) -> Result<()> {
        require_gte!(
            self.accumulated_deposit_capacity_amount,
            token_amount,
            ErrorCode::FundInvalidUpdateError
        );

        self.accumulated_deposit_amount = token_amount;

        Ok(())
    }

    pub(super) fn set_accumulated_deposit_capacity_amount(
        &mut self,
        token_amount: u64,
    ) -> Result<()> {
        require_gte!(
            token_amount,
            self.accumulated_deposit_amount,
            ErrorCode::FundInvalidUpdateError
        );

        self.accumulated_deposit_capacity_amount = token_amount;

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
            .accumulated_deposit_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        require_gte!(
            self.accumulated_deposit_capacity_amount,
            new_accumulated_deposit_amount,
            ErrorCode::FundExceededTokenCapacityAmountError
        );

        self.accumulated_deposit_amount = new_accumulated_deposit_amount;
        self.operation_reserved_amount = self
            .operation_reserved_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }
}
