use anchor_lang::prelude::*;

use crate::{error::ErrorCode, fund::*};

impl FundV1 {
    pub(super) fn update_token(&mut self, token: Pubkey, info: TokenInfo) -> Result<()> {
        let token_info = self
            .whitelisted_tokens
            .iter_mut()
            .find(|info| info.address == token)
            .ok_or(ErrorCode::FundNotExistingToken)?;

        *token_info = info;

        Ok(())
    }

    pub(super) fn add_whitelisted_token(&mut self, token: Pubkey, token_cap: u128) -> Result<()> {
        self.check_if_token_exists(&token)?;

        let token_info = TokenInfo::empty(token, token_cap);
        self.whitelisted_tokens.push(token_info);

        Ok(())
    }

    fn check_if_token_exists(&self, token: &Pubkey) -> Result<()> {
        if self
            .whitelisted_tokens
            .iter()
            .any(|info| info.address == *token)
        {
            return Err(ErrorCode::FundAlreadyExistingToken)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_whitelisted_token() {
        let mut fund = FundV1 {
            default_protocol_fee_rate: 0,
            whitelisted_tokens: vec![],
            sol_amount_in: 0,
        };

        let token1 = TokenInfo {
            address: Pubkey::new_unique(),
            token_cap: 1_000_000_000 * 1000,
            token_amount_in: 1_000_000_000,
        };
        let token2 = TokenInfo {
            address: Pubkey::new_unique(),
            token_cap: 1_000_000_000 * 2000,
            token_amount_in: 2_000_000_000,
        };
        let token3 = token1.clone();
        let tokens = vec![token1, token2];

        fund.whitelisted_tokens = tokens;
        fund.add_whitelisted_token(token3.address, token3.token_cap)
            .unwrap();
    }

    #[test]
    fn test_update_token() {
        let default_protocol_fee_rate = 10;

        let mut fund = FundV1 {
            default_protocol_fee_rate: 0,
            whitelisted_tokens: vec![],
            sol_amount_in: 0,
        };

        let token1 = TokenInfo {
            address: Pubkey::new_unique(),
            token_cap: 1_000_000_000 * 1000,
            token_amount_in: 1_000_000_000,
        };
        let token2 = TokenInfo {
            address: Pubkey::new_unique(),
            token_cap: 1_000_000_000 * 2000,
            token_amount_in: 2_000_000_000,
        };
        let mut token1_update = token1.clone();
        token1_update.token_cap = 1_000_000_000 * 3000;
        let tokens = vec![token1, token2];

        fund.initialize(default_protocol_fee_rate, tokens).unwrap();
        println!("{:?}", fund.whitelisted_tokens.iter());

        fund.update_token(token1_update.address, token1_update)
            .unwrap();
        println!("{:?}", fund.whitelisted_tokens.iter());
    }
}
