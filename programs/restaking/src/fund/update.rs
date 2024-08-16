use anchor_lang::prelude::*;

use crate::{error::ErrorCode, fund::*};

impl TokenInfo {
    pub(super) fn update(&mut self, token_cap: u64) {
        self.token_cap = token_cap;
    }
}

impl Fund {
    pub(super) fn add_whitelisted_token(&mut self, token: Pubkey, token_cap: u64) {
        let token_info = TokenInfo::empty(token, token_cap);
        self.whitelisted_tokens.push(token_info);
    }

    pub(super) fn check_token_does_not_exist(&self, token: &Pubkey) -> Result<()> {
        if self
            .whitelisted_tokens
            .iter()
            .any(|info| info.address == *token)
        {
            err!(ErrorCode::FundAlreadyExistingToken)?
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_duplicated_token_fails() {
        let mut fund = Fund {
            admin: Pubkey::default(),
            receipt_token_mint: Pubkey::default(),
            whitelisted_tokens: vec![],
            sol_amount_in: 0,
            withdrawal_status: Default::default(),
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
        fund.check_token_does_not_exist(&token3.address)
            .unwrap_err();
    }

    #[test]
    fn test_update_token() {
        let mut fund = Fund {
            admin: Pubkey::default(),
            receipt_token_mint: Pubkey::default(),
            whitelisted_tokens: vec![],
            sol_amount_in: 0,
            withdrawal_status: Default::default(),
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

        fund.set_whitelisted_tokens(tokens);
        println!("{:?}", fund.whitelisted_tokens.iter());

        fund.whitelisted_token_mut(token1_update.address)
            .unwrap()
            .update(token1_update.token_cap);
        println!("{:?}", fund.whitelisted_tokens.iter());
    }
}
