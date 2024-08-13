use anchor_lang::prelude::*;

use crate::{error::ErrorCode, fund::*};

impl Fund {
    pub(super) fn deposit_token(&mut self, token: Pubkey, amount: u64) -> Result<u64> {
        let token_info = self
            .whitelisted_tokens
            .iter_mut()
            .find(|info| info.address == token)
            .ok_or_else(|| error!(ErrorCode::FundNotExistingToken))?;

        let new_token_amount_in = token_info
            .token_amount_in
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
        if token_info.token_cap < new_token_amount_in {
            err!(ErrorCode::FundExceedsTokenCap)?
        }

        token_info.token_amount_in = new_token_amount_in;

        Ok(token_info.token_amount_in)
    }

    pub(super) fn deposit_sol(&mut self, amount: u64) -> Result<()> {
        self.sol_amount_in = self
            .sol_amount_in
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;

        Ok(())
    }
}
