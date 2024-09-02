use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::fund::{FundAccount, SupportedTokenInfo, TokenPricingSource, UserFundAccount, WithdrawalStatus};

impl SupportedTokenInfo {
    pub fn set_capacity_amount(&mut self, capacity_amount: u64) -> Result<()> {
        if self.capacity_amount < self.accumulated_deposit_amount {
            err!(ErrorCode::FundInvalidUpdateError)?
        }
        self.capacity_amount = capacity_amount;

        Ok(())
    }
}

impl FundAccount {
    pub fn set_sol_capacity_amount(&mut self, capacity_amount: u64) -> Result<()> {
        if self.sol_capacity_amount < self.sol_accumulated_deposit_amount {
            err!(ErrorCode::FundInvalidUpdateError)?
        }
        self.sol_capacity_amount = capacity_amount;

        Ok(())
    }

    pub fn add_supported_token(
        &mut self,
        mint: Pubkey,
        decimals: u8,
        capacity_amount: u64,
        pricing_source: TokenPricingSource,
        pricing_sources: &[&AccountInfo],
    ) -> Result<()> {
        if self.supported_tokens.iter().any(|info| info.mint == mint) {
            err!(ErrorCode::FundAlreadySupportedTokenError)?
        }
        let token_info = SupportedTokenInfo::new(
            mint,
            decimals,
            capacity_amount,
            pricing_source
        );
        self.supported_tokens.push(token_info);
        self.update_token_prices(pricing_sources)?;

        Ok(())
    }
}

impl WithdrawalStatus {
    pub fn set_sol_withdrawal_fee_rate(&mut self, sol_withdrawal_fee_rate: u16) {
        self.sol_withdrawal_fee_rate = sol_withdrawal_fee_rate;
    }

    pub fn set_withdrawal_enabled_flag(&mut self, flag: bool) {
        self.withdrawal_enabled_flag = flag;
    }

    pub fn set_batch_processing_threshold(
        &mut self,
        amount: Option<u64>,
        duration: Option<i64>,
    ) {
        if let Some(amount) = amount {
            self.batch_processing_threshold_amount = amount;
        }
        if let Some(duration) = duration {
            self.batch_processing_threshold_duration = duration;
        }
    }
}

impl UserFundAccount {
    pub fn set_receipt_token_amount(&mut self, total_amount: u64) {
        self.receipt_token_amount = total_amount;
    }
}

#[cfg(test)]
mod tests {
    use crate::constants::{BSOL_STAKE_POOL_ADDRESS, MSOL_STAKE_POOL_ADDRESS};
    use crate::modules::fund::{FundAccount, SupportedTokenInfo, TokenPricingSource};
    use super::*;

    #[test]
    fn test_update_token() {
        let mut fund = FundAccount {
            data_version: 1,
            bump: 0,
            receipt_token_mint: Pubkey::default(),
            supported_tokens: vec![],
            sol_capacity_amount: 1_000_000_000 * 10000,
            sol_accumulated_deposit_amount: 0,
            sol_operation_reserved_amount: 0,
            withdrawal_status: Default::default(),
            _reserved: [0; 1280],
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
            _reserved: [0; 128],
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
            _reserved: [0; 128],
        };
        let mut dummy_lamports = 0u64;
        let mut dummy_data: [u8; 0] = [];
        let mut dummy_lamports2 = 0u64;
        let mut dummy_data2: [u8; 0] = [];
        let pricing_sources = &[
            &AccountInfo::new(
                &BSOL_STAKE_POOL_ADDRESS,
                false,
                false,
                &mut dummy_lamports,
                &mut dummy_data,
                &BSOL_STAKE_POOL_ADDRESS,
                false,
                0,
            ),
            &AccountInfo::new(
                &MSOL_STAKE_POOL_ADDRESS,
                false,
                false,
                &mut dummy_lamports2,
                &mut dummy_data2,
                &MSOL_STAKE_POOL_ADDRESS,
                false,
                0,
            ),
        ];
        let mut token1_update = token1.clone();
        token1_update.capacity_amount = 1_000_000_000 * 3000;
        fund.add_supported_token(
            token1.mint,
            token2.decimals,
            token1.capacity_amount,
            token1.pricing_source,
            pricing_sources,
        ).unwrap();
        fund.add_supported_token(
            token2.mint,
            token2.decimals,
            token2.capacity_amount,
            token2.pricing_source,
            pricing_sources,
        ).unwrap();
        println!("{:?}", fund.supported_tokens.iter());

        fund.supported_token_mut(token1_update.mint)
            .unwrap()
            .set_capacity_amount(token1_update.capacity_amount)
            .unwrap();
        println!("{:?}", fund.supported_tokens.iter());
    }
}
