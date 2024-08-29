use anchor_lang::prelude::*;

use crate::{error::ErrorCode, fund::*};

impl SupportedTokenInfo {
    pub(super) fn update(&mut self, capacity_amount: u64) {
        self.capacity_amount = capacity_amount;
    }
}

impl Fund {
    pub(super) fn add_supported_token(
        &mut self,
        mint: Pubkey,
        decimals: u8,
        capacity_amount: u64,
        pricing_source: TokenPricingSource,
    ) {
        let token_info = SupportedTokenInfo::empty(mint, decimals, capacity_amount, pricing_source);
        self.supported_tokens.push(token_info);
    }

    pub(super) fn check_token_does_not_exist(&self, token: &Pubkey) -> Result<()> {
        if self.supported_tokens.iter().any(|info| info.mint == *token) {
            err!(ErrorCode::FundAlreadyExistingToken)?
        }

        Ok(())
    }
}

impl UserReceipt {
    pub(crate) fn set_receipt_token_amount(&mut self, total_amount: u64) {
        self.receipt_token_amount = total_amount;
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
            sol_capacity_amount: 1_000_000_000 * 10000,
            sol_accumulated_deposit_amount: 0,
            sol_operation_reserved_amount: 0,
            withdrawal_status: Default::default(),
        };

        let token1 = SupportedTokenInfo {
            mint: Pubkey::new_unique(),
            decimals: 9,
            capacity_amount: 1_000_000_000 * 1000,
            accumulated_deposit_amount: 0,
            operation_reserved_amount: 1_000_000_000,
            price: 0,
            pricing_source: TokenPricingSource::SPLStakePool {
                address: Pubkey::new_unique(),
            },
        };
        let token2 = SupportedTokenInfo {
            mint: Pubkey::new_unique(),
            decimals: 9,
            capacity_amount: 1_000_000_000 * 2000,
            accumulated_deposit_amount: 0,
            operation_reserved_amount: 2_000_000_000,
            price: 0,
            pricing_source: TokenPricingSource::SPLStakePool {
                address: Pubkey::new_unique(),
            },
        };
        let token3 = token1.clone();
        let tokens = vec![token1, token2];

        fund.supported_tokens = tokens;
        fund.check_token_does_not_exist(&token3.mint).unwrap_err();
    }

    #[test]
    fn test_update_token() {
        let mut fund = Fund {
            data_version: 1,
            bump: 0,
            receipt_token_mint: Pubkey::default(),
            supported_tokens: vec![],
            sol_capacity_amount: 1_000_000_000 * 10000,
            sol_accumulated_deposit_amount: 0,
            sol_operation_reserved_amount: 0,
            withdrawal_status: Default::default(),
        };

        let token1 = SupportedTokenInfo {
            mint: Pubkey::new_unique(),
            decimals: 9,
            capacity_amount: 1_000_000_000 * 1000,
            accumulated_deposit_amount: 0,
            operation_reserved_amount: 1_000_000_000,
            price: 0,
            pricing_source: TokenPricingSource::SPLStakePool {
                address: Pubkey::new_unique(),
            },
        };
        let token2 = SupportedTokenInfo {
            mint: Pubkey::new_unique(),
            decimals: 9,
            capacity_amount: 1_000_000_000 * 2000,
            accumulated_deposit_amount: 0,
            operation_reserved_amount: 2_000_000_000,
            price: 0,
            pricing_source: TokenPricingSource::SPLStakePool {
                address: Pubkey::new_unique(),
            },
        };
        let mut token1_update = token1.clone();
        token1_update.capacity_amount = 1_000_000_000 * 3000;
        fund.add_supported_token(
            token1.mint,
            token1.decimals,
            token1.capacity_amount,
            token1.pricing_source,
        );
        fund.add_supported_token(
            token2.mint,
            token2.decimals,
            token2.capacity_amount,
            token2.pricing_source,
        );
        println!("{:?}", fund.supported_tokens.iter());

        fund.supported_token_mut(token1_update.mint)
            .unwrap()
            .update(token1_update.capacity_amount);
        println!("{:?}", fund.supported_tokens.iter());
    }
}
