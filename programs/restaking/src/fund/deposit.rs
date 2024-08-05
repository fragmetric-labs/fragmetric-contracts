use anchor_lang::prelude::*;

use crate::{error::ErrorCode, fund::*};

impl FundV2 {
    pub(super) fn deposit_token(&mut self, token: Pubkey, amount: u64) -> Result<()> {
        let token_info = self
            .whitelisted_tokens
            .iter_mut()
            .find(|info| info.address == token)
            .ok_or_else(|| error!(ErrorCode::FundNotExistingToken))?;

        if token_info.token_cap < token_info.token_amount_in + amount as u128 {
            err!(ErrorCode::FundExceedsTokenCap)?
        }

        token_info.token_amount_in += amount as u128;

        Ok(())
    }

    pub(super) fn deposit_sol(&mut self, amount: u64) -> Result<()> {
        self.sol_amount_in += amount as u128;

        Ok(())
    }
}
