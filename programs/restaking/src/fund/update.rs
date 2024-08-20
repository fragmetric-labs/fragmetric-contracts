use anchor_lang::prelude::*;

use crate::{error::ErrorCode, fund::*};

impl TokenInfo {
    pub(super) fn update(&mut self, token_cap: u64) {
        self.token_cap = token_cap;
    }
}

impl Fund {
    pub(super) fn add_supported_token(
        &mut self,
        token: Pubkey,
        token_decimal: u8,
        token_cap: u64,
        pricing_source: PricingSource,
    ) {
        let token_info = TokenInfo::empty(token, token_decimal, token_cap, pricing_source);
        self.supported_tokens.push(token_info);
    }

    pub(super) fn check_token_does_not_exist(&self, token: &Pubkey) -> Result<()> {
        if self
            .supported_tokens
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
            data_version: 1,
            bump: 0,
            receipt_token_mint: Pubkey::default(),
            supported_tokens: vec![],
            sol_amount_in: 0,
            withdrawal_status: Default::default(),
        };

        let token1 = TokenInfo {
            address: Pubkey::new_unique(),
            token_decimal: 9,
            token_cap: 1_000_000_000 * 1000,
            token_amount_in: 1_000_000_000,
            token_price: 0,
            pricing_source: PricingSource::SPLStakePool {
                address: Pubkey::new_unique(),
            },
        };
        let token2 = TokenInfo {
            address: Pubkey::new_unique(),
            token_decimal: 9,
            token_cap: 1_000_000_000 * 2000,
            token_amount_in: 2_000_000_000,
            token_price: 0,
            pricing_source: PricingSource::SPLStakePool {
                address: Pubkey::new_unique(),
            },
        };
        let token3 = token1.clone();
        let tokens = vec![token1, token2];

        fund.supported_tokens = tokens;
        fund.check_token_does_not_exist(&token3.address)
            .unwrap_err();
    }

    #[test]
    fn test_update_token() {
        let mut fund = Fund {
            data_version: 1,
            bump: 0,
            receipt_token_mint: Pubkey::default(),
            supported_tokens: vec![],
            sol_amount_in: 0,
            withdrawal_status: Default::default(),
        };

        let token1 = TokenInfo {
            address: Pubkey::new_unique(),
            token_decimal: 9,
            token_cap: 1_000_000_000 * 1000,
            token_amount_in: 1_000_000_000,
            token_price: 0,
            pricing_source: PricingSource::SPLStakePool {
                address: Pubkey::new_unique(),
            },
        };
        let token2 = TokenInfo {
            address: Pubkey::new_unique(),
            token_decimal: 9,
            token_cap: 1_000_000_000 * 2000,
            token_amount_in: 2_000_000_000,
            token_price: 0,
            pricing_source: PricingSource::SPLStakePool {
                address: Pubkey::new_unique(),
            },
        };
        let mut token1_update = token1.clone();
        token1_update.token_cap = 1_000_000_000 * 3000;
        fund.add_supported_token(
            token1.address,
            token1.token_decimal,
            token1.token_cap,
            token1.pricing_source,
        );
        fund.add_supported_token(
            token2.address,
            token2.token_decimal,
            token2.token_cap,
            token2.pricing_source,
        );
        println!("{:?}", fund.supported_tokens.iter());

        fund.supported_token_mut(token1_update.address)
            .unwrap()
            .update(token1_update.token_cap);
        println!("{:?}", fund.supported_tokens.iter());
    }
}
