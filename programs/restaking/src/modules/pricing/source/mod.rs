use std::collections::BTreeMap;

use anchor_lang::prelude::*;

mod marinade_stake_pool;
#[cfg(test)]
mod mock;
mod normalized_token_pool;
mod spl_stake_pool;

use anchor_spl::token_interface::Mint;
use marinade_stake_pool::*;
#[cfg(test)]
pub(super) use mock::*;
use normalized_token_pool::*;
use spl_stake_pool::*;

use crate::errors::ErrorCode;
use crate::modules::normalize::NormalizedTokenPoolConfig;

/// { mint : (pricing_source, account list) }
pub type TokenPricingSourceMap<'info> = BTreeMap<
    Pubkey, // mint as key
    (TokenPricingSource, Vec<&'info AccountInfo<'info>>),
>;

/// A type that can calculate the token amount as sol with its data.
pub(super) trait TokenAmountAsSOLCalculator<'info> {
    fn calculate_token_amount_as_sol(
        &self,
        token_amount: u64,
        pricing_source_map: &TokenPricingSourceMap<'info>,
    ) -> Result<u64>;
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
#[non_exhaustive]
pub enum TokenPricingSource {
    SPLStakePool {
        address: Pubkey,
    },
    MarinadeStakePool {
        address: Pubkey,
    },
    NormalizedTokenPool {
        mint: Pubkey,
        config: Pubkey,
    },
    #[cfg(test)]
    Mock,
}

pub(super) fn create_token_amount_as_sol_calculator<'info>(
    mint: Pubkey,
    pricing_source_map: &TokenPricingSourceMap<'info>,
) -> Result<Box<dyn TokenAmountAsSOLCalculator<'info> + 'info>> {
    let (pricing_source, pricing_source_accounts) = pricing_source_map
        .get(&mint)
        .ok_or_else(|| error!(ErrorCode::FundTokenPricingSourceNotFoundException))?;
    match pricing_source {
        TokenPricingSource::SPLStakePool { .. } => {
            require_eq!(pricing_source_accounts.len(), 1);

            Ok(Box::new(
                Account::<SplStakePool>::try_from(pricing_source_accounts[0])?.into_inner(),
            ))
        }
        TokenPricingSource::MarinadeStakePool { .. } => {
            require_eq!(pricing_source_accounts.len(), 1);

            Ok(Box::new(
                Account::<MarinadeStakePool>::try_from(pricing_source_accounts[0])?.into_inner(),
            ))
        }
        TokenPricingSource::NormalizedTokenPool { .. } => {
            require_eq!(pricing_source_accounts.len(), 2);

            let normalized_token_pool_config =
                Account::<NormalizedTokenPoolConfig>::try_from(pricing_source_accounts[0])?;
            let normalized_token_mint =
                InterfaceAccount::<Mint>::try_from(pricing_source_accounts[1])?;

            Ok(Box::new(NormalizedTokenAmountAsSOLCalculator::new(
                normalized_token_pool_config,
                normalized_token_mint,
            )))
        }
        #[cfg(test)]
        TokenPricingSource::Mock => Ok(Box::new(MockPriceSource)),
    }
}
