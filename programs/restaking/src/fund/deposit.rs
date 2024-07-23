use anchor_lang::prelude::*;

use crate::fund::*;
use crate::error::ErrorCode;

impl Fund {
    pub fn deposit_token(&mut self, token: Pubkey, amount: u64) -> Result<()> {
        for mapped_token in self.tokens.iter_mut() {
            if mapped_token.address == token {
                if mapped_token.token_cap < (mapped_token.token_amount_in + amount as u128) as u64 {
                    return Err(ErrorCode::ExceedsTokenCap)?;
                }
                mapped_token.token_amount_in += amount as u128;
                return Ok(())
            }
        }
        err!(ErrorCode::NotExistingToken)
    }

    pub fn deposit_sol(&mut self, amount: u64) -> Result<()> {
        self.sol_amount_in += amount as u128;
    
        Ok(())
    }
}
