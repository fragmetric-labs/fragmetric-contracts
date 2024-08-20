use anchor_lang::prelude::*;

use crate::{error::ErrorCode, fund::*};

impl TokenInfo {
    pub(super) fn deposit_token(&mut self, amount: u64) -> Result<()> {
        let new_token_amount_in = self
            .token_amount_in
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
        if self.token_cap < new_token_amount_in {
            err!(ErrorCode::FundExceedsTokenCap)?
        }

        let new_token_to_sol_value = self
            .token_to_sol_value
            .checked_add(self.calculate_sol_from_tokens(amount)?)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;

        self.token_amount_in = new_token_amount_in;
        self.token_to_sol_value = new_token_to_sol_value;

        Ok(())
    }
}

impl Fund {
    pub(super) fn deposit_sol(&mut self, amount: u64) -> Result<()> {
        self.sol_amount_in = self
            .sol_amount_in
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Metadata {
    pub wallet_provider: String,
    pub fpoint_accrual_rate_multiplier: f32,
}
