use std::collections::BTreeMap;

use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::fund::FundReceiptTokenValueProvider;
use crate::modules::normalization::NormalizedTokenPoolValueProvider;
use crate::modules::staking::{MarinadeStakePoolValueProvider, SPLStakePoolValueProvider};
use crate::utils;

#[cfg(test)]
use super::MockPricingSourceValueProvider;
use super::{Asset, TokenPricingSource, TokenValue, TokenValueProvider};

pub struct PricingService<'info> {
    token_pricing_source_accounts_map: BTreeMap<Pubkey, AccountInfo<'info>>,
    token_value_map: BTreeMap<Pubkey, TokenValue>,
}

impl<'info> PricingService<'info> {
    pub fn new(token_pricing_source_accounts: &[AccountInfo<'info>]) -> Result<Self> {
        Ok(Self {
            token_pricing_source_accounts_map: token_pricing_source_accounts
                .iter()
                .map(|account| (account.key(), account.clone()))
                .collect(),
            token_value_map: BTreeMap::new(),
        })
    }

    pub fn register_token_pricing_source_account(
        &mut self,
        token_pricing_source_account: &AccountInfo<'info>,
    ) -> &mut Self {
        self.token_pricing_source_accounts_map.insert(
            token_pricing_source_account.key(),
            token_pricing_source_account.clone(),
        );
        self
    }

    pub fn resolve_token_pricing_source(
        &mut self,
        token_mint: &Pubkey,
        token_pricing_source: &TokenPricingSource,
    ) -> Result<&mut Self> {
        let token_value = match token_pricing_source {
            TokenPricingSource::SPLStakePool { address } => {
                let account1 = self
                    .token_pricing_source_accounts_map
                    .get(address)
                    .ok_or_else(|| error!(ErrorCode::TokenPricingSourceAccountNotFoundException))?;
                SPLStakePoolValueProvider.resolve_underlying_assets(vec![account1])?
            }
            TokenPricingSource::MarinadeStakePool { address } => {
                let account1 = self
                    .token_pricing_source_accounts_map
                    .get(address)
                    .ok_or_else(|| error!(ErrorCode::TokenPricingSourceAccountNotFoundException))?;
                MarinadeStakePoolValueProvider.resolve_underlying_assets(vec![account1])?
            }
            TokenPricingSource::NormalizedTokenPool {
                mint_address,
                pool_address,
            } => {
                let account1 = self
                    .token_pricing_source_accounts_map
                    .get(mint_address)
                    .ok_or_else(|| error!(ErrorCode::TokenPricingSourceAccountNotFoundException))?;
                let account2 = self
                    .token_pricing_source_accounts_map
                    .get(pool_address)
                    .ok_or_else(|| error!(ErrorCode::TokenPricingSourceAccountNotFoundException))?;
                NormalizedTokenPoolValueProvider
                    .resolve_underlying_assets(vec![account1, account2])?
            }
            TokenPricingSource::FundReceiptToken {
                mint_address,
                fund_address,
            } => {
                let account1 = self
                    .token_pricing_source_accounts_map
                    .get(mint_address)
                    .ok_or_else(|| error!(ErrorCode::TokenPricingSourceAccountNotFoundException))?;
                let account2 = self
                    .token_pricing_source_accounts_map
                    .get(fund_address)
                    .ok_or_else(|| error!(ErrorCode::TokenPricingSourceAccountNotFoundException))?;
                FundReceiptTokenValueProvider.resolve_underlying_assets(vec![account1, account2])?
            }
            #[cfg(test)]
            TokenPricingSource::Mock {
                numerator,
                denominator,
            } => MockPricingSourceValueProvider::new(numerator, denominator)
                .resolve_underlying_assets(vec![])?,
        };

        // expand supported tokens recursively
        token_value.numerator.iter().try_for_each(|asset| {
            if let Asset::TOKEN(token_mint, Some(token_pricing_source), _) = asset {
                self.resolve_token_pricing_source(token_mint, token_pricing_source)?;
            }
            Ok::<(), Error>(())
        })?;

        // check already registered token
        if let Some(registered_token_value) = self.token_value_map.get(token_mint) {
            require_eq!(registered_token_value, &token_value);
            return Ok(self);
        }

        // store resolved token value
        // if *token_mint == FRAGSOL_MINT_ADDRESS {
        //     msg!("PRICING: {:?} => {:?}", token_mint, token_value);
        // }
        self.token_value_map.insert(*token_mint, token_value);
        Ok(self)
    }

    // returns (total sol value of the token, total token amount)
    fn resolve_token_total_value_as_sol(&self, token_mint: &Pubkey) -> Result<(u64, u64)> {
        let token_value = self
            .token_value_map
            .get(token_mint)
            .ok_or_else(|| error!(ErrorCode::TokenPricingSourceAccountNotFoundException))?;
        let mut total_sol_amount = 0u64;

        for asset in &token_value.numerator {
            match asset {
                Asset::SOL(sol_amount) => {
                    total_sol_amount = total_sol_amount
                        .checked_add(*sol_amount)
                        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
                }
                Asset::TOKEN(nested_token_mint, _, nested_token_amount) => {
                    let nested_sol_amount =
                        self.get_token_amount_as_sol(nested_token_mint, *nested_token_amount)?;
                    total_sol_amount = total_sol_amount
                        .checked_add(nested_sol_amount)
                        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
                }
            }
        }

        Ok((total_sol_amount, token_value.denominator))
    }

    pub fn get_sol_amount_as_token(&self, token_mint: &Pubkey, sol_amount: u64) -> Result<u64> {
        let (total_token_value_as_sol, total_token_amount) =
            self.resolve_token_total_value_as_sol(token_mint)?;
        let token_amount = utils::get_proportional_amount(
            sol_amount,
            total_token_amount,
            total_token_value_as_sol,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        // if *token_mint == FRAGSOL_MINT_ADDRESS {
        //     msg!("PRICING: {} SOL => {} {:?} ({} SOL / {} TOKEN)", sol_amount, token_amount, token_mint, total_token_value_as_sol, total_token_amount);
        // }
        Ok(token_amount)
    }

    pub fn get_token_amount_as_sol(&self, token_mint: &Pubkey, token_amount: u64) -> Result<u64> {
        let (total_token_value_as_sol, total_token_amount) =
            self.resolve_token_total_value_as_sol(token_mint)?;
        let sol_amount = utils::get_proportional_amount(
            token_amount,
            total_token_value_as_sol,
            total_token_amount,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        // if *token_mint == FRAGSOL_MINT_ADDRESS {
        //     msg!("PRICING: {} SOL <= {} {:?} ({} SOL / {} TOKEN)", sol_amount, token_amount, token_mint, total_token_value_as_sol, total_token_amount);
        // }
        Ok(sol_amount)
    }
}

#[cfg(test)]
mod tests {
    use crate::modules::pricing::MockAsset;

    use super::*;

    #[test]
    fn test_mock_pricing_source() {
        let mut pricing_service = PricingService::new(&[]).unwrap();

        let mock_mint_10_10 = Pubkey::new_unique();
        pricing_service
            .resolve_token_pricing_source(
                &mock_mint_10_10,
                &TokenPricingSource::Mock {
                    numerator: vec![MockAsset::SOL(10)],
                    denominator: 10,
                },
            )
            .unwrap();
        assert_eq!(
            pricing_service
                .get_sol_amount_as_token(&mock_mint_10_10, 1_000)
                .unwrap(),
            1_000
        );
        assert_eq!(
            pricing_service
                .get_token_amount_as_sol(&mock_mint_10_10, 2_000)
                .unwrap(),
            2_000
        );

        let mock_mint_12_10 = Pubkey::new_unique();
        pricing_service
            .resolve_token_pricing_source(
                &mock_mint_12_10,
                &TokenPricingSource::Mock {
                    numerator: vec![
                        MockAsset::SOL(10_000),
                        MockAsset::TOKEN(mock_mint_10_10, 2_000),
                    ],
                    denominator: 10_000,
                },
            )
            .unwrap();
        assert_eq!(
            pricing_service
                .get_sol_amount_as_token(&mock_mint_12_10, 1_200)
                .unwrap(),
            1_000
        );
        assert_eq!(
            pricing_service
                .get_token_amount_as_sol(&mock_mint_12_10, 2_000)
                .unwrap(),
            2_400
        );

        let mock_mint_14_10 = Pubkey::new_unique();
        let mock_source_14_10 = &TokenPricingSource::Mock {
            numerator: vec![
                MockAsset::SOL(2_000),
                MockAsset::TOKEN(mock_mint_12_10, 10_000),
            ],
            denominator: 10_000,
        };
        pricing_service
            .resolve_token_pricing_source(&mock_mint_10_10, mock_source_14_10)
            .map(|_| ())
            .expect_err("resolve_token_pricing_source fails for already registered token");
        pricing_service
            .resolve_token_pricing_source(&mock_mint_14_10, mock_source_14_10)
            .unwrap();
        assert_eq!(
            pricing_service
                .get_sol_amount_as_token(&mock_mint_14_10, 1_400)
                .unwrap(),
            1_000
        );
        assert_eq!(
            pricing_service
                .get_token_amount_as_sol(&mock_mint_14_10, 2_000)
                .unwrap(),
            2_800
        );
    }
}
