use anchor_lang::prelude::*;

mod marinade_stake_pool;
mod spl_stake_pool;

use marinade_stake_pool::*;
use spl_stake_pool::*;

use crate::errors::ErrorCode;

/// A type that can calculate the token price with its data.
pub trait TokenPriceCalculator {
    fn calculate_token_price(&self, token_amount: u64) -> Result<u64>;
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
#[non_exhaustive]
pub enum TokenPricingSource {
    SPLStakePool { address: Pubkey },
    MarinadeStakePool { address: Pubkey },
}

pub struct TokenPriceCalculatorFactory;

impl TokenPriceCalculatorFactory {
    pub fn to_calculator_checked<'info>(
        &self,
        source: &TokenPricingSource,
        accounts: &'info [AccountInfo<'info>],
    ) -> Result<Box<dyn TokenPriceCalculator>> {
        match source {
            TokenPricingSource::SPLStakePool { address } => {
                let account = self.find_token_pricing_source_by_key(accounts, address)?;
                Ok(Box::new(
                    Account::<SplStakePool>::try_from(account)?.into_inner(),
                ))
            }
            TokenPricingSource::MarinadeStakePool { address } => {
                let account = self.find_token_pricing_source_by_key(accounts, address)?;
                Ok(Box::new(
                    Account::<MarinadeStakePool>::try_from(account)?.into_inner(),
                ))
            }
        }
    }

    fn find_token_pricing_source_by_key<'info>(
        &self,
        accounts: &'info [AccountInfo<'info>],
        key: &Pubkey,
    ) -> Result<&'info AccountInfo<'info>> {
        accounts
            .iter()
            .find(|account| account.key == key)
            .ok_or_else(|| error!(ErrorCode::FundTokenPricingSourceNotFoundException))
    }
}
