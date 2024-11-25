use std::collections::BTreeMap;

use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::fund::FundReceiptTokenValueProvider;
use crate::modules::normalization::NormalizedTokenPoolValueProvider;
use crate::modules::restaking::JitoRestakingVaultValueProvider;
use crate::modules::staking::{MarinadeStakePoolValueProvider, SPLStakePoolValueProvider};
use crate::utils;

#[cfg(test)]
use super::MockPricingSourceValueProvider;
use super::{Asset, TokenPricingSource, TokenValue, TokenValueProvider};

pub struct PricingService<'info> {
    token_pricing_source_accounts_map: BTreeMap<Pubkey, &'info AccountInfo<'info>>,
    token_pricing_source_map: BTreeMap<Pubkey, TokenPricingSource>,
    /// the boolean flag indicates "atomic", meaning whether the token is not a kind of basket such as normalized token, so the value of the token can be resolved by one self.
    token_value_map: BTreeMap<Pubkey, (TokenValue, bool)>,
}

impl<'info> PricingService<'info> {
    pub fn new(
        token_pricing_source_accounts: impl IntoIterator<Item = &'info AccountInfo<'info>>,
    ) -> Result<Self> {
        Ok(Self {
            token_pricing_source_accounts_map: token_pricing_source_accounts
                .into_iter()
                .map(|account| (account.key(), account))
                .collect(),
            token_pricing_source_map: BTreeMap::new(),
            token_value_map: BTreeMap::new(),
        })
    }

    pub fn register_token_pricing_source_account(
        mut self,
        token_pricing_source_account: &'info AccountInfo<'info>,
    ) -> Self {
        self.token_pricing_source_accounts_map.insert(
            token_pricing_source_account.key(),
            token_pricing_source_account,
        );
        self
    }

    pub fn resolve_token_pricing_source(
        &mut self,
        token_mint: &Pubkey,
        token_pricing_source: &TokenPricingSource,
    ) -> Result<()> {
        let token_value = match token_pricing_source {
            TokenPricingSource::SPLStakePool { address } => {
                let account1 = self
                    .token_pricing_source_accounts_map
                    .get(address)
                    .ok_or_else(|| error!(ErrorCode::TokenPricingSourceAccountNotFoundException))?;
                SPLStakePoolValueProvider.resolve_underlying_assets(token_mint, &[account1])?
            }
            TokenPricingSource::MarinadeStakePool { address } => {
                let account1 = self
                    .token_pricing_source_accounts_map
                    .get(address)
                    .ok_or_else(|| error!(ErrorCode::TokenPricingSourceAccountNotFoundException))?;
                MarinadeStakePoolValueProvider.resolve_underlying_assets(token_mint, &[account1])?
            }
            TokenPricingSource::JitoRestakingVault { address } => {
                let account1 = self
                    .token_pricing_source_accounts_map
                    .get(address)
                    .ok_or_else(|| error!(ErrorCode::TokenPricingSourceAccountNotFoundException))?;
                JitoRestakingVaultValueProvider.resolve_underlying_assets(token_mint, &[account1])?
            }
            TokenPricingSource::FragmetricNormalizedTokenPool { address } => {
                let account1 = self
                    .token_pricing_source_accounts_map
                    .get(address)
                    .ok_or_else(|| error!(ErrorCode::TokenPricingSourceAccountNotFoundException))?;
                NormalizedTokenPoolValueProvider
                    .resolve_underlying_assets(token_mint, &[account1])?
            }
            TokenPricingSource::FragmetricRestakingFund { address } => {
                let account1 = self
                    .token_pricing_source_accounts_map
                    .get(address)
                    .ok_or_else(|| error!(ErrorCode::TokenPricingSourceAccountNotFoundException))?;
                FundReceiptTokenValueProvider.resolve_underlying_assets(token_mint, &[account1])?
            }
            #[cfg(test)]
            TokenPricingSource::Mock {
                numerator,
                denominator,
            } => MockPricingSourceValueProvider::new(numerator, denominator)
                .resolve_underlying_assets(token_mint, &[])?,
        };

        // expand supported tokens recursively
        let mut atomic = true;
        token_value.numerator.iter().try_for_each(|asset| {
            if let Asset::TOKEN(..) = asset {
                atomic = false;
            }
            if let Asset::TOKEN(token_mint, Some(token_pricing_source), _) = asset {
                self.resolve_token_pricing_source(token_mint, token_pricing_source)?;
            }
            Ok::<(), Error>(())
        })?;

        // if *token_mint == FRAGSOL_MINT_ADDRESS {
        //     msg!(
        //         "PRICING: {:?} => {:?} (atomic={})",
        //         token_mint,
        //         token_value,
        //         atomic
        //     );
        // }

        // remember resolved token value and pricing source
        self.token_value_map
            .insert(*token_mint, (token_value, atomic));
        if !self.token_pricing_source_map.contains_key(token_mint) {
            self.token_pricing_source_map
                .insert(*token_mint, token_pricing_source.clone());
        }

        Ok(())
    }

    /// returns (total sol value of the token, total token amount)
    fn get_token_total_value_as_sol(&self, token_mint: &Pubkey) -> Result<(u64, u64)> {
        let (token_value, _) = self
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
            self.get_token_total_value_as_sol(token_mint)?;
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
            self.get_token_total_value_as_sol(token_mint)?;
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

    /// returns token value being consist of atomic tokens, either SOL or LSTs
    pub fn get_token_total_value_as_atomic(&self, token_mint: &Pubkey) -> Result<TokenValue> {
        let (token_value, token_atomic) = self
            .token_value_map
            .get(token_mint)
            .ok_or_else(|| error!(ErrorCode::TokenPricingSourceAccountNotFoundException))?;

        if *token_atomic {
            return Ok(token_value.clone());
        }

        let mut total_tokens: BTreeMap<Pubkey, u64> = BTreeMap::new();
        let mut total_sol_amount = 0u64;

        for asset in &token_value.numerator {
            match asset {
                Asset::SOL(sol_amount) => {
                    total_sol_amount += sol_amount;
                }
                Asset::TOKEN(token_mint, _, token_amount) => {
                    let (_, token_atomic) =
                        self.token_value_map.get(token_mint).ok_or_else(|| {
                            error!(ErrorCode::TokenPricingSourceAccountNotFoundException)
                        })?;

                    if *token_atomic {
                        total_tokens.insert(
                            *token_mint,
                            total_tokens.get(token_mint).unwrap_or(&0u64) + token_amount,
                        );
                    } else {
                        let nested_token_value =
                            self.get_token_total_value_as_atomic(token_mint)?;
                        for nested_asset in &nested_token_value.numerator {
                            match nested_asset {
                                Asset::SOL(nested_sol_amount) => {
                                    total_sol_amount += utils::get_proportional_amount(
                                        *nested_sol_amount,
                                        *token_amount,
                                        nested_token_value.denominator,
                                    )
                                    .ok_or_else(|| {
                                        error!(ErrorCode::CalculationArithmeticException)
                                    })?;
                                }
                                Asset::TOKEN(nested_token_mint, _, nested_token_amount) => {
                                    total_tokens.insert(
                                        *nested_token_mint,
                                        total_tokens.get(nested_token_mint).unwrap_or(&0u64)
                                            + utils::get_proportional_amount(
                                                *nested_token_amount,
                                                *token_amount,
                                                nested_token_value.denominator,
                                            )
                                            .ok_or_else(|| {
                                                error!(ErrorCode::CalculationArithmeticException)
                                            })?,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut numerator = Vec::new();
        if total_sol_amount > 0 {
            numerator.push(Asset::SOL(total_sol_amount));
        }
        numerator.extend(
            total_tokens
                .into_iter()
                .filter(|(_, token_amount)| *token_amount > 0)
                .map(|(token_mint, token_amount)| {
                    Asset::TOKEN(
                        token_mint,
                        self.token_pricing_source_map.get(&token_mint).cloned(),
                        token_amount,
                    )
                }),
        );

        Ok(TokenValue {
            numerator,
            denominator: token_value.denominator,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::modules::pricing::MockAsset;

    use super::*;

    #[test]
    fn test_resolve_token_pricing_source() {
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
                        MockAsset::Token(mock_mint_10_10, 2_000),
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
                MockAsset::Token(mock_mint_12_10, 10_000),
            ],
            denominator: 10_000,
        };
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

        let mock_source_14_10_updated = &TokenPricingSource::Mock {
            numerator: vec![
                MockAsset::SOL(4_000),
                MockAsset::Token(mock_mint_12_10, 20_000),
            ],
            denominator: 10_000,
        };
        pricing_service
            .resolve_token_pricing_source(&mock_mint_14_10, mock_source_14_10_updated)
            .unwrap();
        assert_eq!(
            pricing_service
                .get_sol_amount_as_token(&mock_mint_14_10, 1_400)
                .unwrap(),
            500
        );
        assert_eq!(
            pricing_service
                .get_token_amount_as_sol(&mock_mint_14_10, 2_000)
                .unwrap(),
            5_600
        );
    }

    #[test]
    fn test_resolve_token_total_value_as_atomic() {
        let mut pricing_service = PricingService::new(&[]).unwrap();

        let atomic_mint_10_10 = Pubkey::new_unique();
        pricing_service
            .resolve_token_pricing_source(
                &atomic_mint_10_10,
                &TokenPricingSource::Mock {
                    numerator: vec![MockAsset::SOL(10)],
                    denominator: 10,
                },
            )
            .unwrap();

        let atomic_mint_12_10 = Pubkey::new_unique();
        pricing_service
            .resolve_token_pricing_source(
                &atomic_mint_12_10,
                &TokenPricingSource::Mock {
                    numerator: vec![MockAsset::SOL(12)],
                    denominator: 10,
                },
            )
            .unwrap();

        let atomic_mint_16_10 = Pubkey::new_unique();
        pricing_service
            .resolve_token_pricing_source(
                &atomic_mint_16_10,
                &TokenPricingSource::Mock {
                    numerator: vec![MockAsset::SOL(16)],
                    denominator: 10,
                },
            )
            .unwrap();

        let basket_mint_28_10 = Pubkey::new_unique();
        pricing_service
            .resolve_token_pricing_source(
                &basket_mint_28_10,
                &TokenPricingSource::Mock {
                    numerator: vec![
                        MockAsset::SOL(20),
                        MockAsset::Token(atomic_mint_10_10, 5),
                        MockAsset::Token(atomic_mint_12_10, 5),
                        MockAsset::Token(atomic_mint_16_10, 5),
                    ],
                    denominator: 10,
                },
            )
            .unwrap();

        let basket_mint_24_10 = Pubkey::new_unique();
        pricing_service
            .resolve_token_pricing_source(
                &basket_mint_24_10,
                &TokenPricingSource::Mock {
                    numerator: vec![MockAsset::SOL(10), MockAsset::Token(basket_mint_28_10, 10)],
                    denominator: 10,
                },
            )
            .unwrap();

        let token_vaule_as_atomic = pricing_service
            .get_token_total_value_as_atomic(&basket_mint_24_10)
            .unwrap();
        let token_value_as_sol = pricing_service
            .get_token_total_value_as_sol(&basket_mint_24_10)
            .unwrap();

        // println!("{:?} / {:?}", token_vaule_as_atomic, token_value_as_sol,);

        assert_eq!(token_value_as_sol.0, 49);
        assert_eq!(token_value_as_sol.1, 10);
        assert_eq!(token_vaule_as_atomic.denominator, token_value_as_sol.1);
        assert_eq!(token_vaule_as_atomic.denominator, token_value_as_sol.1);
        let mut total_tokens_as_sol = 0;
        for asset in &token_vaule_as_atomic.numerator {
            match asset {
                Asset::SOL(sol_amount) => {
                    assert_eq!(*sol_amount, 30); // 20 + 10
                }
                Asset::TOKEN(token_mint, pricing_source, token_amount) => {
                    assert_ne!(*pricing_source, None);
                    assert_eq!(*token_amount, 5);
                    total_tokens_as_sol += pricing_service
                        .get_token_amount_as_sol(token_mint, *token_amount)
                        .unwrap();
                }
            }
        }
        assert_eq!(total_tokens_as_sol, 19);
    }
}
