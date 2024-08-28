use anchor_lang::prelude::*;

use crate::{error::ErrorCode, fund::*};

impl SupportedTokenInfo {
    pub(super) fn deposit_token(&mut self, amount: u64) -> Result<()> {
        let new_operation_reserved_amount = self
            .operation_reserved_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
        if self.capacity_amount < new_operation_reserved_amount {
            err!(ErrorCode::FundExceedsTokenCap)?
        }

        self.operation_reserved_amount = new_operation_reserved_amount;

        Ok(())
    }
}

impl Fund {
    pub(super) fn deposit_sol(&mut self, amount: u64) -> Result<()> {
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
