use anchor_lang::prelude::*;

use crate::{error::ErrorCode, fund::*};

impl SupportedTokenInfo {
    pub(super) fn deposit_token(&mut self, amount: u64) -> Result<()> {
        let new_accumulated_deposit_amount = self
            .accumulated_deposit_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
        if self.capacity_amount < new_accumulated_deposit_amount {
            err!(ErrorCode::FundExceedsTokenCap)?
        }

        self.accumulated_deposit_amount = new_accumulated_deposit_amount;
        self.operation_reserved_amount = self
            .operation_reserved_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;

        Ok(())
    }
}

impl Fund {
    pub(super) fn deposit_sol(&mut self, amount: u64) -> Result<()> {
        let new_sol_accumulated_deposit_amount = self
            .sol_accumulated_deposit_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
        if self.sol_capacity_amount < new_sol_accumulated_deposit_amount {
            err!(ErrorCode::FundExceedsSolCap)?
        }

        self.sol_accumulated_deposit_amount = new_sol_accumulated_deposit_amount;
        self.sol_operation_reserved_amount = self
            .sol_operation_reserved_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
        if self.sol_capacity_amount < new_sol_accumulated_deposit_amount {
            err!(ErrorCode::FundExceedsSolCap)?
        }

        self.sol_accumulated_deposit_amount = new_sol_accumulated_deposit_amount;
        self.sol_operation_reserved_amount = self
            .sol_operation_reserved_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Metadata {
    pub wallet_provider: String,
    pub contribution_accrual_rate: f32,
}
