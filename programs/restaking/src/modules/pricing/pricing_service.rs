use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::fund::FundReceiptTokenValueProvider;
use crate::modules::normalization::NormalizedTokenPoolValueProvider;
use crate::modules::restaking::JitoRestakingVaultValueProvider;
use crate::modules::staking::{MarinadeStakePoolValueProvider, SPLStakePoolValueProvider};
use crate::modules::swap::OrcaDEXLiquidityPoolValueProvider;
use crate::utils;

#[cfg(all(test, not(feature = "idl-build")))]
use super::MockPricingSourceValueProvider;
use super::{Asset, TokenPricingSource, TokenValue, TokenValueProvider};

const PRICING_SERVICE_EXPECTED_TOKENS_SIZE: usize = 32;

pub(in crate::modules) struct PricingService<'info> {
    token_pricing_sources_account_infos: Vec<&'info AccountInfo<'info>>,
    token_mints: Vec<Pubkey>,
    token_pricing_sources: Vec<TokenPricingSource>,
    token_values: Vec<TokenValue>,
}

impl<'info> PricingService<'info> {
    pub fn new<I>(token_pricing_source_accounts: I) -> Result<Self>
    where
        I: IntoIterator<Item = &'info AccountInfo<'info>>,
        I::IntoIter: ExactSizeIterator,
    {
        Ok(Self {
            token_pricing_sources_account_infos: token_pricing_source_accounts
                .into_iter()
                .collect(),
            token_mints: Vec::with_capacity(PRICING_SERVICE_EXPECTED_TOKENS_SIZE),
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
        address: &Pubkey,
    ) -> Result<&'info AccountInfo<'info>> {
        self.token_pricing_sources_account_infos
            .iter()
            .find(|account| account.key == address)
            .copied()
            .ok_or_else(|| error!(ErrorCode::TokenPricingSourceAccountNotFoundError))
    }

    fn get_token_index(&self, mint: &Pubkey) -> Option<usize> {
        self.token_mints.iter().position(|key| key == mint)
    }

    fn get_token_pricing_source(&self, mint: &Pubkey) -> Option<&TokenPricingSource> {
        Some(&self.token_pricing_sources[self.get_token_index(mint)?])
    }

    fn get_token_value(&self, mint: &Pubkey) -> Result<&TokenValue> {
        let index = self
            .get_token_index(mint)
            .ok_or_else(|| error!(ErrorCode::TokenPricingSourceAccountNotFoundError))?;
        Ok(&self.token_values[index])
    }

    pub fn resolve_token_pricing_source(
        &mut self,
        token_mint: &Pubkey,
        token_pricing_source: &TokenPricingSource,
    ) -> Result<()> {
        self.resolve_token_pricing_source_rec(token_mint, token_pricing_source, &mut 0)
    }

    fn resolve_token_pricing_source_rec(
        &mut self,
        token_mint: &Pubkey,
        token_pricing_source: &TokenPricingSource,
        updated_token_values_index_bitmap: &mut u64,
    ) -> Result<()> {
        // remember indices of updated token values during recursion to skip redundant calculation
        let token_index = if let Some(index) = self.get_token_index(token_mint) {
            // token pricing source should be same
            #[cfg(not(test))]
            require_eq!(token_pricing_source, &self.token_pricing_sources[index]);

            if *updated_token_values_index_bitmap & (1 << index) > 0 {
                return Ok(());
            }

            index
        } else {
            self.token_mints.push(*token_mint);
            self.token_pricing_sources
                .push(token_pricing_source.clone());
            // First we just push dummy TokenValue, and it will be updated soon!!
            self.token_values.push(TokenValue::default());
            self.token_mints.len() - 1
        };
        *updated_token_values_index_bitmap |= 1 << token_index;

        // resolve underlying assets for each pricing source' value provider adapter
        match token_pricing_source {
            TokenPricingSource::SPLStakePool { address } => {
                let pricing_source_accounts =
                    [self.get_token_pricing_source_account_info(address)?];
                SPLStakePoolValueProvider.resolve_underlying_assets(
                    token_mint,
                    &pricing_source_accounts,
                    &mut self.token_values[token_index],
                )?
            }
            TokenPricingSource::MarinadeStakePool { address } => {
                let pricing_source_accounts =
                    [self.get_token_pricing_source_account_info(address)?];
                MarinadeStakePoolValueProvider.resolve_underlying_assets(
                    token_mint,
                    &pricing_source_accounts,
                    &mut self.token_values[token_index],
                )?
            }
            TokenPricingSource::JitoRestakingVault { address } => {
                let pricing_source_accounts =
                    [self.get_token_pricing_source_account_info(address)?];
                JitoRestakingVaultValueProvider.resolve_underlying_assets(
                    token_mint,
                    &pricing_source_accounts,
                    &mut self.token_values[token_index],
                )?
            }
            TokenPricingSource::FragmetricNormalizedTokenPool { address } => {
                let pricing_source_accounts =
                    [self.get_token_pricing_source_account_info(address)?];
                NormalizedTokenPoolValueProvider.resolve_underlying_assets(
                    token_mint,
                    &pricing_source_accounts,
                    &mut self.token_values[token_index],
                )?
            }
            TokenPricingSource::FragmetricRestakingFund { address } => {
                let pricing_source_accounts =
                    [self.get_token_pricing_source_account_info(address)?];
                FundReceiptTokenValueProvider.resolve_underlying_assets(
                    token_mint,
                    &pricing_source_accounts,
                    &mut self.token_values[token_index],
                )?
            }
            TokenPricingSource::OrcaDEXLiquidityPool { address } => {
                let pricing_source_accounts =
                    [self.get_token_pricing_source_account_info(address)?];
                OrcaDEXLiquidityPoolValueProvider.resolve_underlying_assets(
                    token_mint,
                    &pricing_source_accounts,
                    &mut self.token_values[token_index],
                )?
            }
            TokenPricingSource::SanctumSingleValidatorSPLStakePool { address } => {
                let pricing_source_accounts =
                    [self.get_token_pricing_source_account_info(address)?];
                SPLStakePoolValueProvider.resolve_underlying_assets(
                    token_mint,
                    &pricing_source_accounts,
                    &mut self.token_values[token_index],
                )?
            }
            #[cfg(all(test, not(feature = "idl-build")))]
            TokenPricingSource::Mock {
                numerator,
                denominator,
            } => MockPricingSourceValueProvider::new(numerator, denominator)
                .resolve_underlying_assets(token_mint, &[], &mut self.token_values[token_index])?,
        };

        // expand supported tokens recursively
        // due to ownership, first we take numerator and return it back after recursion
        let assets = std::mem::take(&mut self.token_values[token_index].numerator);
        for asset in &assets {
            if let Asset::Token(token_mint, Some(token_pricing_source), _) = asset {
                self.resolve_token_pricing_source_rec(
                    token_mint,
                    token_pricing_source,
                    updated_token_values_index_bitmap,
                )?;
            }
        }
        self.token_values[token_index].numerator = assets;

        Ok(())
    }

    /// returns the token value in (numerator_as_sol, denominator_as_token)
    fn get_token_value_as_sol(&self, token_mint: &Pubkey) -> Result<(u64, u64)> {
        let token_value = self.get_token_value(token_mint)?;
        let mut total_sol_amount = 0u64;

        for asset in &token_value.numerator {
            match asset {
                Asset::SOL(sol_amount) => {
                    total_sol_amount += sol_amount;
                }
                Asset::Token(nested_token_mint, _, nested_token_amount) => {
                    total_sol_amount +=
                        self.get_token_amount_as_sol(nested_token_mint, *nested_token_amount)?;
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

    /// **Flatten**s the token value of given token.
    /// A token value is **flattened** if and only if:
    /// * there is no duplicated assets in token value.
    /// * all assets in token value are atomic tokens.
    /// * all assets in token value have positive amount.
    pub fn flatten_token_value(&self, token_mint: &Pubkey, result: &mut TokenValue) -> Result<()> {
        let token_value = self.get_token_value(token_mint)?;

        result.numerator.clear();
        result.denominator = token_value.denominator;

        // If token value is atomic, then it is already summarized.
        if token_value.is_atomic() {
            let num_assets = token_value.numerator.len();
            result.numerator.reserve_exact(num_assets);

            result.numerator.extend_from_slice(&token_value.numerator);

            return Ok(());
        }

        result.numerator.reserve_exact(1);

        for asset in &token_value.numerator {
            match asset {
                Asset::SOL(sol_amount) => {
                    self.flatten_token_value_of_sol(*sol_amount, result);
                }
                Asset::Token(token_mint, token_pricing_source, token_amount) => {
                    self.flatten_token_value_of_token(
                        token_mint,
                        token_pricing_source.as_ref(),
                        *token_amount,
                        result,
                    )?;
                }
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn flatten_token_value_of_sol(&self, sol_amount: u64, result: &mut TokenValue) {
        if sol_amount > 0 {
            result.add_sol(sol_amount);
        }
    }

    /// Recursively traverse the assets of token value of token and flattens into atomic assets.
    fn flatten_token_value_of_token(
        &self,
        token_mint: &Pubkey,
        token_pricing_source: Option<&TokenPricingSource>,
        token_amount: u64,
        result: &mut TokenValue,
    ) -> Result<()> {
        if token_amount == 0 {
            return Ok(());
        }

        let token_value = self.get_token_value(token_mint)?;
        if token_value.is_atomic() {
            result.add_token(
                token_mint,
                token_pricing_source.or_else(|| self.get_token_pricing_source(token_mint)),
                token_amount,
            );
            return Ok(());
        }

        for nested_asset in &token_value.numerator {
            match nested_asset {
                Asset::SOL(nested_sol_amount) => {
                    let nested_sol_amount = utils::get_proportional_amount(
                        *nested_sol_amount,
                        token_amount,
                        token_value.denominator,
                    )?;
                    self.flatten_token_value_of_sol(nested_sol_amount, result);
                }
                Asset::Token(
                    nested_token_mint,
                    nested_token_pricing_source,
                    nested_token_amount,
                ) => {
                    let nested_token_amount = utils::get_proportional_amount(
                        *nested_token_amount,
                        token_amount,
                        token_value.denominator,
                    )?;
                    self.flatten_token_value_of_token(
                        nested_token_mint,
                        nested_token_pricing_source.as_ref(),
                        nested_token_amount,
                        result,
                    )?;
                }
            }
        }

        Ok(())
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

        let basket_mint_39_10 = Pubkey::new_unique();
        pricing_service
            .resolve_token_pricing_source(
                &basket_mint_39_10,
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

        let basket_mint_49_10 = Pubkey::new_unique();
        pricing_service
            .resolve_token_pricing_source(
                &basket_mint_49_10,
                &TokenPricingSource::Mock {
                    numerator: vec![MockAsset::SOL(10), MockAsset::Token(basket_mint_39_10, 10)],
                    denominator: 10,
                },
            )
            .unwrap();

        let mut token_value_as_atomic = TokenValue::default();
        pricing_service
            .flatten_token_value(&basket_mint_49_10, &mut token_value_as_atomic)
            .unwrap();
        let token_value_as_sol = pricing_service
            .get_token_value_as_sol(&basket_mint_49_10)
            .unwrap();

        assert_eq!(
            format!("{:?}", token_value_as_atomic),
            format!(
                "TokenValue {{ numerator: [SOL(30), Token({:?}, Some(Mock {{ numerator: [SOL(10)], denominator: 10 }}), 5), Token({:?}, Some(Mock {{ numerator: [SOL(12)], denominator: 10 }}), 5), Token({:?}, Some(Mock {{ numerator: [SOL(16)], denominator: 10 }}), 5)], denominator: 10 }}",
                atomic_mint_10_10,
                atomic_mint_12_10,
                atomic_mint_16_10,
            ),
        );

        assert_eq!(token_value_as_sol.0, 49);
        assert_eq!(token_value_as_sol.1, 10);
        assert_eq!(token_value_as_atomic.denominator, token_value_as_sol.1);
        let mut total_tokens_as_sol = 0;
        for asset in &token_value_as_atomic.numerator {
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

        let basket_mint_88_10 = Pubkey::new_unique();
        pricing_service
            .resolve_token_pricing_source(
                &basket_mint_88_10,
                &TokenPricingSource::Mock {
                    numerator: vec![MockAsset::SOL(10), MockAsset::Token(basket_mint_39_10, 20)],
                    denominator: 10,
                },
            )
            .unwrap();

        let mut token_value_as_atomic = TokenValue::default();
        pricing_service
            .flatten_token_value(&basket_mint_88_10, &mut token_value_as_atomic)
            .unwrap();
        let token_value_as_sol = pricing_service
            .get_token_value_as_sol(&basket_mint_88_10)
            .unwrap();

        assert_eq!(
                format!("{:?}", token_value_as_atomic),
                format!(
                    "TokenValue {{ numerator: [SOL(50), Token({:?}, Some(Mock {{ numerator: [SOL(10)], denominator: 10 }}), 10), Token({:?}, Some(Mock {{ numerator: [SOL(12)], denominator: 10 }}), 10), Token({:?}, Some(Mock {{ numerator: [SOL(16)], denominator: 10 }}), 10)], denominator: 10 }}",
                    atomic_mint_10_10,
                    atomic_mint_12_10,
                    atomic_mint_16_10,
                ),
            );

        assert_eq!(token_value_as_sol.0, 88);
        assert_eq!(token_value_as_sol.1, 10);
        assert_eq!(token_value_as_atomic.denominator, token_value_as_sol.1);
        let mut total_tokens_as_sol = 0;
        for asset in &token_value_as_atomic.numerator {
            match asset {
                Asset::SOL(sol_amount) => {
                    assert_eq!(*sol_amount, 50); // 40 + 10
                }
                Asset::Token(token_mint, pricing_source, token_amount) => {
                    assert_ne!(*pricing_source, None);
                    assert_eq!(*token_amount, 10);
                    total_tokens_as_sol += pricing_service
                        .get_token_amount_as_sol(token_mint, *token_amount)
                        .unwrap();
                }
            }
        }
        assert_eq!(total_tokens_as_sol, 38);
    }
}
