use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::fund::FundReceiptTokenValueProvider;
use crate::modules::normalization::NormalizedTokenPoolValueProvider;
use crate::modules::restaking::JitoRestakingVaultValueProvider;
use crate::modules::staking::{MarinadeStakePoolValueProvider, SPLStakePoolValueProvider};
use crate::modules::swap::OrcaLiqPoolValueProvider;
use crate::utils;

#[cfg(all(test, not(feature = "idl-build")))]
use super::MockPricingSourceValueProvider;
use super::{Asset, TokenPricingSource, TokenValue, TokenValuePod, TokenValueProvider};

pub const PRICING_SERVICE_EXPECTED_TOKENS_SIZE: usize = 16;

pub struct PricingService<'info> {
    token_pricing_sources_account_infos: Vec<&'info AccountInfo<'info>>,
    token_pricing_sources: Vec<(Pubkey, TokenPricingSource)>,
    token_values: Vec<(Pubkey, TokenValue)>,
}

impl<'info> PricingService<'info> {
    pub fn new(
        token_pricing_source_accounts: impl IntoIterator<Item = &'info AccountInfo<'info>>,
    ) -> Result<Self> {
        Ok(Self {
            token_pricing_sources_account_infos: token_pricing_source_accounts
                .into_iter()
                .collect(),
            token_pricing_sources: Vec::with_capacity(PRICING_SERVICE_EXPECTED_TOKENS_SIZE),
            token_values: Vec::with_capacity(PRICING_SERVICE_EXPECTED_TOKENS_SIZE),
        })
    }

    pub fn register_token_pricing_source_account(
        mut self,
        token_pricing_source_account: &'info AccountInfo<'info>,
    ) -> Self {
        if !self
            .token_pricing_sources_account_infos
            .iter()
            .any(|account| token_pricing_source_account.key() == *account.key)
        {
            self.token_pricing_sources_account_infos
                .push(token_pricing_source_account);
        }
        self
    }

    fn get_token_pricing_source_account_info(
        &self,
        mint: &Pubkey,
    ) -> Result<&'info AccountInfo<'info>> {
        Ok(self
            .token_pricing_sources_account_infos
            .iter()
            .find(|account| account.key == mint)
            .copied()
            .ok_or_else(|| error!(ErrorCode::TokenPricingSourceAccountNotFoundError))?)
    }

    fn get_token_pricing_source(&self, mint: &Pubkey) -> Option<&TokenPricingSource> {
        self.token_pricing_sources
            .iter()
            .find(|(key, _)| key == mint)
            .map(|(_, source)| source)
    }

    fn get_token_value(&self, mint: &Pubkey) -> Option<&TokenValue> {
        self.token_values
            .iter()
            .find(|(key, _)| key == mint)
            .map(|(_, value)| value)
    }

    fn get_token_value_mut(&mut self, mint: &Pubkey) -> Option<&mut TokenValue> {
        self.token_values
            .iter_mut()
            .find(|(key, _)| key == mint)
            .map(|(_, value)| value)
    }

    pub fn resolve_token_pricing_source(
        &mut self,
        token_mint: &Pubkey,
        token_pricing_source: &TokenPricingSource,
    ) -> Result<()> {
        let updated_tokens = &mut Vec::with_capacity(1);
        self.resolve_token_pricing_source_rec(token_mint, token_pricing_source, updated_tokens)
    }

    fn resolve_token_pricing_source_rec(
        &mut self,
        token_mint: &Pubkey,
        token_pricing_source: &TokenPricingSource,
        updated_tokens: &mut Vec<Pubkey>,
    ) -> Result<()> {
        // remember updated token during the current recursive updates to skip redundant calculation
        if updated_tokens.contains(token_mint) {
            return Ok(());
        }
        updated_tokens.push(*token_mint);

        // resolve underlying assets for each pricing source' value provider adapter
        let token_value = match token_pricing_source {
            TokenPricingSource::SPLStakePool { address } => SPLStakePoolValueProvider
                .resolve_underlying_assets(
                    token_mint,
                    &[self.get_token_pricing_source_account_info(address)?],
                )?,
            TokenPricingSource::MarinadeStakePool { address } => MarinadeStakePoolValueProvider
                .resolve_underlying_assets(
                    token_mint,
                    &[self.get_token_pricing_source_account_info(address)?],
                )?,
            TokenPricingSource::JitoRestakingVault { address } => JitoRestakingVaultValueProvider
                .resolve_underlying_assets(
                token_mint,
                &[self.get_token_pricing_source_account_info(address)?],
            )?,
            TokenPricingSource::FragmetricNormalizedTokenPool { address } => {
                NormalizedTokenPoolValueProvider.resolve_underlying_assets(
                    token_mint,
                    &[self.get_token_pricing_source_account_info(address)?],
                )?
            }
            TokenPricingSource::FragmetricRestakingFund { address } => {
                FundReceiptTokenValueProvider.resolve_underlying_assets(
                    token_mint,
                    &[self.get_token_pricing_source_account_info(address)?],
                )?
            }
            TokenPricingSource::OrcaSOLBaseLiqPool { address } => OrcaLiqPoolValueProvider
                .resolve_underlying_assets(
                    token_mint,
                    &[self.get_token_pricing_source_account_info(address)?],
                )?,
            #[cfg(all(test, not(feature = "idl-build")))]
            TokenPricingSource::Mock {
                numerator,
                denominator,
            } => MockPricingSourceValueProvider::new(numerator, denominator)
                .resolve_underlying_assets(token_mint, &[])?,
        };

        // expand supported tokens recursively
        for asset in token_value.numerator.iter() {
            if let Asset::Token(token_mint, Some(token_pricing_source), _) = asset {
                self.resolve_token_pricing_source_rec(
                    token_mint,
                    token_pricing_source,
                    updated_tokens,
                )?;
            }
        }

        // update resolved token value
        match self.get_token_value_mut(token_mint) {
            Some(old_token_value) => *old_token_value = token_value,
            None => self.token_values.push((*token_mint, token_value)),
        };

        // remember new pricing source
        match self.get_token_pricing_source(token_mint) {
            #[allow(unused_variables)]
            Some(old_token_pricing_source) => {
                #[cfg(not(test))]
                require_eq!(token_pricing_source, old_token_pricing_source);
            }
            None => {
                self.token_pricing_sources
                    .push((*token_mint, token_pricing_source.clone()));
            }
        }

        Ok(())
    }

    /// returns the token value in (numerator_as_sol, denominator_as_token)
    fn get_token_value_as_sol(&self, token_mint: &Pubkey) -> Result<(u64, u64)> {
        let token_value = self
            .get_token_value(token_mint)
            .ok_or_else(|| error!(ErrorCode::TokenPricingSourceAccountNotFoundError))?;
        let mut total_sol_amount = 0u64;

        for asset in &token_value.numerator {
            match asset {
                Asset::SOL(sol_amount) => {
                    total_sol_amount = total_sol_amount
                        .checked_add(*sol_amount)
                        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
                }
                Asset::Token(nested_token_mint, _, nested_token_amount) => {
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
        let (numerator_as_sol, denominator_as_token) = self.get_token_value_as_sol(token_mint)?;
        utils::get_proportional_amount(sol_amount, denominator_as_token, numerator_as_sol)
    }

    pub fn get_token_amount_as_sol(&self, token_mint: &Pubkey, token_amount: u64) -> Result<u64> {
        let (numerator_as_sol, denominator_as_token) = self.get_token_value_as_sol(token_mint)?;
        utils::get_proportional_amount(token_amount, numerator_as_sol, denominator_as_token)
    }

    /// returns token value being consist of atomic tokens, either SOL or LSTs
    pub fn get_token_total_value_as_atomic(&self, token_mint: &Pubkey) -> Result<TokenValue> {
        let token_value = self
            .get_token_value(token_mint)
            .ok_or_else(|| error!(ErrorCode::TokenPricingSourceAccountNotFoundError))?;

        if token_value.is_atomic() {
            return Ok(token_value.clone());
        }

        let mut atomic_token_value = TokenValue {
            numerator: Vec::with_capacity(1),
            denominator: token_value.denominator,
        };

        for asset in &token_value.numerator {
            match asset {
                Asset::SOL(sol_amount) => {
                    if *sol_amount > 0 {
                        atomic_token_value.add(asset.clone());
                    }
                }
                Asset::Token(token_mint, _, token_amount) => {
                    if *token_amount > 0 {
                        let is_token_atomic = self
                            .get_token_value(token_mint)
                            .ok_or_else(|| {
                                error!(ErrorCode::TokenPricingSourceAccountNotFoundError)
                            })?
                            .is_atomic();
                        if is_token_atomic {
                            atomic_token_value.add(asset.clone());
                        } else {
                            let nested_token_value =
                                self.get_token_total_value_as_atomic(token_mint)?;
                            for nested_asset in &nested_token_value.numerator {
                                match nested_asset {
                                    Asset::SOL(nested_sol_amount) => {
                                        let proportional_sol_amount =
                                            utils::get_proportional_amount(
                                                *nested_sol_amount,
                                                *token_amount,
                                                nested_token_value.denominator,
                                            )?;

                                        if proportional_sol_amount > 0 {
                                            atomic_token_value
                                                .add(Asset::SOL(proportional_sol_amount));
                                        }
                                    }
                                    Asset::Token(
                                        nested_token_mint,
                                        nested_pricing_source,
                                        nested_token_amount,
                                    ) => {
                                        let proportional_token_amount =
                                            utils::get_proportional_amount(
                                                *nested_token_amount,
                                                *token_amount,
                                                nested_token_value.denominator,
                                            )?;

                                        if proportional_token_amount > 0 {
                                            atomic_token_value.add(Asset::Token(
                                                *nested_token_mint,
                                                nested_pricing_source.clone().or_else(|| {
                                                    self.get_token_pricing_source(nested_token_mint)
                                                        .cloned()
                                                }),
                                                proportional_token_amount,
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(atomic_token_value)
    }
}

#[cfg(all(test, not(feature = "idl-build")))]
mod tests {
    use crate::modules::pricing::MockAsset;

    use super::*;

    #[test]
    fn size_token_pricing_source() {
        println!(
            "token pricing source init size: {}",
            TokenPricingSource::INIT_SPACE
        );
    }

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

        let atomic_mint_10_10 = pubkey!("bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1");
        pricing_service
            .resolve_token_pricing_source(
                &atomic_mint_10_10,
                &TokenPricingSource::Mock {
                    numerator: vec![MockAsset::SOL(10)],
                    denominator: 10,
                },
            )
            .unwrap();

        let atomic_mint_12_10 = pubkey!("mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So");
        pricing_service
            .resolve_token_pricing_source(
                &atomic_mint_12_10,
                &TokenPricingSource::Mock {
                    numerator: vec![MockAsset::SOL(12)],
                    denominator: 10,
                },
            )
            .unwrap();

        let atomic_mint_16_10 = pubkey!("J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn");
        pricing_service
            .resolve_token_pricing_source(
                &atomic_mint_16_10,
                &TokenPricingSource::Mock {
                    numerator: vec![MockAsset::SOL(16)],
                    denominator: 10,
                },
            )
            .unwrap();

        let basket_mint_28_10 = pubkey!("nSoLnkrvh2aY792pgCNT6hzx84vYtkviRzxvhf3ws8e");
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

        let basket_mint_24_10 = pubkey!("nSoL2krvh2aY792pgCNT6hzx84vYtkviRzxvhf3ws8e");
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
            .get_token_value_as_sol(&basket_mint_24_10)
            .unwrap();

        assert_eq!(format!("{:?}", token_vaule_as_atomic), "TokenValue { numerator: [SOL(30), Token(bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1, Some(Mock { numerator: [SOL(10)], denominator: 10 }), 5), Token(mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So, Some(Mock { numerator: [SOL(12)], denominator: 10 }), 5), Token(J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn, Some(Mock { numerator: [SOL(16)], denominator: 10 }), 5)], denominator: 10 }");

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
                Asset::Token(token_mint, pricing_source, token_amount) => {
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
