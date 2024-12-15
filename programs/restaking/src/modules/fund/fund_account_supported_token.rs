use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::errors::ErrorCode;
use crate::modules::pricing::{TokenPricingSource, TokenPricingSourcePod};

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

    /// configurations: token deposit & withdraw
    pub accumulated_deposit_capacity_amount: u64,
    pub accumulated_deposit_amount: u64,
    _padding2: [u8; 5],
    pub withdrawable: u8,
    pub normal_reserve_rate_bps: u16,
    pub normal_reserve_max_amount: u64,

    /// informative: reserved amount that users can claim for processed withdrawal requests, which is not accounted for as an asset of the fund.
    pub withdrawal_user_reserved_amount: u64,

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

        Ok(())
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

    #[inline(always)]
    pub(super) fn set_normal_reserve_max_amount(&mut self, token_amount: u64) {
        self.normal_reserve_max_amount = token_amount;
    }

    pub(super) fn set_normal_reserve_rate_bps(&mut self, reserve_rate_bps: u16) -> Result<()> {
        require_gte!(
            10_00, // 10%
            reserve_rate_bps,
            ErrorCode::FundInvalidUpdateError
        );

        self.normal_reserve_rate_bps = reserve_rate_bps;

        Ok(())
    }

    #[inline(always)]
    pub(super) fn set_withdrawable(&mut self, withdrawable: bool) {
        self.withdrawable = if withdrawable { 1 } else { 0 };
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
