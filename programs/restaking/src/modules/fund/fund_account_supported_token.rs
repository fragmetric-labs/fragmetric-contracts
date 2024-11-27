use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::pricing::TokenPricingSource;

use super::FundAccount;

// TODO v0.3/operation: visibility
#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct SupportedToken {
    pub mint: Pubkey,
    pub program: Pubkey,
    pub decimals: u8,
    pub capacity_amount: u64,
    pub accumulated_deposit_amount: u64,
    pub operation_reserved_amount: u64,
    pub one_token_as_sol: u64,
    pub pricing_source: TokenPricingSource,

    pub operating_amount: u64,
    /// the amount being unstaked

    /// configuration: the amount requested to be unstaked as soon as possible regardless of current state, this value should be decreased by each unstaking requested amount.
    pub rebalancing_amount: u64,

    /// configuration: used for staking allocation strategy.
    pub sol_allocation_weight: u64,
    pub sol_allocation_capacity_amount: u64,

    _reserved: [u8; 96],
}

impl SupportedToken {
    pub fn new(
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        capacity_amount: u64,
        pricing_source: TokenPricingSource,
    ) -> Result<Self> {
        match pricing_source {
            TokenPricingSource::SPLStakePool { .. }
            | TokenPricingSource::MarinadeStakePool { .. } => {}
            _ => {
                err!(ErrorCode::FundNotSupportedTokenError)?;
            }
        }

        Ok(Self {
            mint,
            program,
            decimals,
            capacity_amount,
            accumulated_deposit_amount: 0,
            operation_reserved_amount: 0,
            one_token_as_sol: 0,
            pricing_source,
            operating_amount: 0,
            rebalancing_amount: 0,
            sol_allocation_weight: 0,
            sol_allocation_capacity_amount: 0,
            _reserved: [0; 96],
        })
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
        self.operating_amount
    }

    // TODO v0.3/operation: visibility
    pub(in crate::modules) fn set_operating_amount(&mut self, amount: u64) {
        self.operating_amount = amount;
    }

    pub(super) fn set_capacity_amount(&mut self, capacity_amount: u64) -> anchor_lang::Result<()> {
        require_gte!(
            capacity_amount,
            self.accumulated_deposit_amount,
            ErrorCode::FundInvalidUpdateError
        );

        self.capacity_amount = capacity_amount;

        Ok(())
    }

    pub(super) fn deposit_token(&mut self, amount: u64) -> anchor_lang::Result<()> {
        let new_accumulated_deposit_amount = self
            .accumulated_deposit_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        require_gte!(
            self.capacity_amount,
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
