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
    /// `token_pricing_source_accounts` must provide exact size_hint.
    pub fn new(
        token_pricing_source_accounts: impl IntoIterator<Item = &'info AccountInfo<'info>>,
    ) -> Self {
        Self {
            token_pricing_sources_account_infos: token_pricing_source_accounts
                .into_iter()
                .collect(),
            token_mints: Vec::with_capacity(PRICING_SERVICE_EXPECTED_TOKENS_SIZE),
            token_pricing_sources: Vec::with_capacity(PRICING_SERVICE_EXPECTED_TOKENS_SIZE),
            token_values: Vec::with_capacity(PRICING_SERVICE_EXPECTED_TOKENS_SIZE),
        }
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

            if *updated_token_values_index_bitmap & (1 << index) != 0 {
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
            TokenPricingSource::PeggedToken { address } => {
                require_keys_neq!(*address, *token_mint);
                self.token_values[token_index] = self.get_token_value(address)?.clone();
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

    /// returns (from_asset_amount, to_token_amount) for the given pair, e.g. returns (1, 1) on 1:1, returns (15, 10) on 1.5:1
    pub fn get_asset_exchange_ratio(
        &self,
        from_asset_mint: Option<&Pubkey>,
        to_token_mint: &Pubkey,
    ) -> Result<Option<(u64, u64)>> {
        match from_asset_mint {
            Some(from_token_mint) => {
                let mut from_token_value = TokenValue::default();
                self.flatten_token_value(from_token_mint, &mut from_token_value, true)?;
                if from_token_value.denominator == 0 {
                    return Ok(None);
                }
                if from_token_value.numerator.len() == 1 {
                    if let Asset::Token(from_nested_token_mint, _, from_nested_token_amount) =
                        &from_token_value.numerator[0]
                    {
                        let to_token_mint_resolved =
                            match self.get_token_pricing_source(to_token_mint) {
                                Some(TokenPricingSource::PeggedToken { address }) => address,
                                None => err!(ErrorCode::TokenPricingSourceAccountNotFoundError)?,
                                _ => to_token_mint,
                            };
                        if from_nested_token_mint == to_token_mint_resolved {
                            return Ok(if *from_nested_token_amount == 0 {
                                None
                            } else {
                                // asking fragJTO : JTO exchange rate,
                                // if from_token_value is like 110JTO (numerator) / 100fragJTO (denominator)
                                // returns (100, 110)
                                Some((from_token_value.denominator, *from_nested_token_amount))
                            });
                        }
                    }
                }

                let mut to_token_value = TokenValue::default();
                self.flatten_token_value(to_token_mint, &mut to_token_value, true)?;
                if to_token_value.denominator == 0 {
                    return Ok(None);
                }
                if to_token_value.numerator.len() == 1 {
                    if let Asset::Token(to_nested_token_mint, _, to_nested_token_amount) =
                        &to_token_value.numerator[0]
                    {
                        let from_token_mint_resolved =
                            match self.get_token_pricing_source(from_token_mint) {
                                Some(TokenPricingSource::PeggedToken { address }) => address,
                                None => err!(ErrorCode::TokenPricingSourceAccountNotFoundError)?,
                                _ => from_token_mint,
                            };
                        if to_nested_token_mint == from_token_mint_resolved {
                            return Ok(if *to_nested_token_amount == 0 {
                                None
                            } else {
                                // asking JTO : fragJTO exchange rate,
                                // if to_token_value is like 110JTO (numerator) / 100fragJTO (denominator)
                                // returns (110, 100)
                                Some((*to_nested_token_amount, to_token_value.denominator))
                            });
                        }
                    }
                }

                let base_amount = 10u64
                    .checked_pow(9)
                    .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
                let from_base_value =
                    self.get_token_amount_as_sol(&from_token_mint, base_amount)?;
                let to_base_value = self.get_token_amount_as_sol(&to_token_mint, base_amount)?;
                Ok(if from_base_value == 0 || to_base_value == 0 {
                    None
                } else {
                    Some((from_base_value, to_base_value))
                })
            }
            None => {
                let base_amount = 10u64
                    .checked_pow(9)
                    .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
                let to_base_value = self.get_token_amount_as_sol(&to_token_mint, base_amount)?;
                Ok(if to_base_value == 0 {
                    None
                } else {
                    Some((base_amount, to_base_value))
                })
            }
        }
    }

    pub fn get_asset_amount_as_token(
        &self,
        from_asset_mint: Option<&Pubkey>,
        from_asset_amount: u64,
        to_token_mint: &Pubkey,
    ) -> Result<u64> {
        match from_asset_mint {
            None => self.get_sol_amount_as_token(to_token_mint, from_asset_amount),
            Some(from_token_mint) => {
                if let Some((from, to)) =
                    self.get_asset_exchange_ratio(from_asset_mint, to_token_mint)?
                {
                    if from == to {
                        return Ok(from_asset_amount);
                    }
                }
                self.get_sol_amount_as_token(
                    to_token_mint,
                    self.get_token_amount_as_sol(from_token_mint, from_asset_amount)?,
                )
            }
        }
    }

    pub fn get_token_amount_as_asset(
        &self,
        from_token_mint: &Pubkey,
        from_token_amount: u64,
        to_asset_mint: Option<&Pubkey>,
    ) -> Result<u64> {
        match to_asset_mint {
            None => self.get_token_amount_as_sol(from_token_mint, from_token_amount),
            Some(to_token_mint) => {
                if let Some((from, to)) =
                    self.get_asset_exchange_ratio(Some(from_token_mint), to_token_mint)?
                {
                    if from == to {
                        return Ok(from_token_amount);
                    }
                }
                self.get_sol_amount_as_token(
                    to_token_mint,
                    self.get_token_amount_as_sol(from_token_mint, from_token_amount)?,
                )
            }
        }
    }

    pub fn get_sol_amount_as_token(&self, to_token_mint: &Pubkey, sol_amount: u64) -> Result<u64> {
        let (numerator_as_sol, denominator_as_token) =
            self.get_token_value_as_sol(to_token_mint)?;
        utils::get_proportional_amount(sol_amount, denominator_as_token, numerator_as_sol)
    }

    pub fn get_token_amount_as_sol(
        &self,
        from_token_mint: &Pubkey,
        token_amount: u64,
    ) -> Result<u64> {
        let (numerator_as_sol, denominator_as_token) =
            self.get_token_value_as_sol(from_token_mint)?;
        utils::get_proportional_amount(token_amount, numerator_as_sol, denominator_as_token)
    }

    pub fn get_one_token_amount_as_sol(
        &self,
        from_token_mint: &Pubkey,
        from_token_decimals: u8,
    ) -> Result<Option<u64>> {
        let (_, denominator_as_token) = self.get_token_value_as_sol(from_token_mint)?;
        Ok(if denominator_as_token == 0 {
            None
        } else {
            Some({
                let token_amount = 10u64
                    .checked_pow(from_token_decimals as u32)
                    .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

                self.get_token_amount_as_asset(from_token_mint, token_amount, None)?
            })
        })
    }

    pub fn get_one_token_amount_as_token(
        &self,
        from_token_mint: &Pubkey,
        from_token_decimals: u8,
        to_token_mint: &Pubkey,
    ) -> Result<Option<u64>> {
        let (_, denominator_as_token1) = self.get_token_value_as_sol(from_token_mint)?;
        let (_, denominator_as_token2) = self.get_token_value_as_sol(to_token_mint)?;
        Ok(
            if denominator_as_token1 == 0 || denominator_as_token2 == 0 {
                None
            } else {
                Some({
                    let token_amount = 10u64
                        .checked_pow(from_token_decimals as u32)
                        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

                    self.get_token_amount_as_asset(
                        from_token_mint,
                        token_amount,
                        Some(to_token_mint),
                    )?
                })
            },
        )
    }

    /// **Flatten**s the token value of given token.
    /// A token value is **flattened** if and only if:
    /// * there is no duplicated assets in token value.
    /// * all assets in token value are atomic tokens.
    /// * all assets in token value have positive amount.
    pub fn flatten_token_value(
        &self,
        token_mint: &Pubkey,
        result: &mut TokenValue,
        merge_pegged_tokens: bool,
    ) -> Result<()> {
        let token_value = self.get_token_value(token_mint)?;

        result.numerator.clear();
        result.denominator = token_value.denominator;

        // If token value is atomic, then it is already summarized.
        if token_value.is_atomic() {
            let num_assets = token_value.numerator.len();
            result.numerator.reserve_exact(num_assets);
            result.numerator.extend_from_slice(&token_value.numerator);
        } else {
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
        }

        // Try to flatten if merge_pegged_tokens is enabled
        if merge_pegged_tokens {
            if let Some(root_mint) = result
                .numerator
                .iter()
                .find_map(|asset| match asset {
                    Asset::Token(_, Some(TokenPricingSource::PeggedToken { address }), _) => {
                        Some(address)
                    }
                    Asset::Token(mint, _, _) => Some(mint),
                    _ => None,
                })
                .cloned()
                .as_ref()
            {
                let mut total_root_amount = 0u64;
                let mut all_pegged = true;

                for asset in result.numerator.iter() {
                    match asset {
                        Asset::Token(mint, pricing_source, amount) => {
                            let resolved_mint = match pricing_source
                                .as_ref()
                                .or_else(|| self.get_token_pricing_source(mint))
                            {
                                Some(TokenPricingSource::PeggedToken { address }) => address,
                                _ => mint,
                            };
                            if root_mint != resolved_mint {
                                all_pegged = false;
                                break;
                            }
                            total_root_amount = total_root_amount
                                .checked_add(*amount)
                                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
                        }
                        Asset::SOL(sol) => {
                            if *sol > 0 {
                                all_pegged = false;
                                break;
                            }
                        }
                    }
                }

                if all_pegged {
                    result.numerator.clear();
                    result.numerator.reserve_exact(1);
                    result.numerator.push(Asset::Token(
                        *root_mint,
                        self.get_token_pricing_source(root_mint).cloned(),
                        total_root_amount,
                    ));
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
    fn test_get_token_exchange_ratio() {
        let mut pricing_service = PricingService::new(&[]);
        let token_mint = Pubkey::new_unique();
        pricing_service
            .resolve_token_pricing_source(
                &token_mint,
                &TokenPricingSource::Mock {
                    numerator: vec![MockAsset::SOL(1_234_567_890 * 2)],
                    denominator: 1_234_567_890,
                },
            )
            .unwrap();

        let stable_fund_mint = Pubkey::new_unique();
        pricing_service
            .resolve_token_pricing_source(
                &stable_fund_mint,
                &TokenPricingSource::Mock {
                    numerator: vec![MockAsset::Token(token_mint, 2_234_567_890)],
                    denominator: 2_234_567_890,
                },
            )
            .unwrap();

        let stable_exchange_rate = pricing_service
            .get_asset_exchange_ratio(Some(&token_mint), &stable_fund_mint)
            .unwrap();
        let (from, to) = stable_exchange_rate.unwrap();
        assert_eq!(from, to);
        assert_eq!(from, 2_234_567_890);

        let stable_exchange_rate_rev = pricing_service
            .get_asset_exchange_ratio(Some(&stable_fund_mint), &token_mint)
            .unwrap();
        let (from, to) = stable_exchange_rate_rev.unwrap();
        assert_eq!(from, to);
        assert_eq!(from, 2_234_567_890);

        let increased_fund_mint = Pubkey::new_unique();
        pricing_service
            .resolve_token_pricing_source(
                &increased_fund_mint,
                &TokenPricingSource::Mock {
                    numerator: vec![MockAsset::Token(token_mint, 2_234_567_890 * 2)],
                    denominator: 2_234_567_890,
                },
            )
            .unwrap();

        let increased_exchange_rate = pricing_service
            .get_asset_exchange_ratio(Some(&token_mint), &increased_fund_mint)
            .unwrap();
        let (from, to) = increased_exchange_rate.unwrap();
        assert!(from > to);
        assert_eq!(from, 2_234_567_890 * 2);
        assert_eq!(to, 2_234_567_890);
        assert_eq!(
            pricing_service
                .get_asset_amount_as_token(Some(&token_mint), 1234, &stable_fund_mint)
                .unwrap(),
            1234
        );
        assert_eq!(
            pricing_service
                .get_asset_amount_as_token(None, 1234, &stable_fund_mint)
                .unwrap(),
            617
        );

        let increased_exchange_rate_rev = pricing_service
            .get_asset_exchange_ratio(Some(&increased_fund_mint), &token_mint)
            .unwrap();
        let (from, to) = increased_exchange_rate_rev.unwrap();
        assert!(from < to);
        assert_eq!(from, 2_234_567_890);
        assert_eq!(to, 2_234_567_890 * 2);
        assert_eq!(
            pricing_service
                .get_asset_amount_as_token(Some(&token_mint), 1234, &increased_fund_mint)
                .unwrap(),
            617
        );
        assert_eq!(
            pricing_service
                .get_asset_amount_as_token(None, 1234, &increased_fund_mint)
                .unwrap(),
            308
        );
    }

    #[test]
    fn test_get_token_exchange_ratio_with_pegged() {
        let mut pricing_service = PricingService::new(&[]);
        let root_token_mint = Pubkey::new_unique();
        pricing_service
            .resolve_token_pricing_source(
                &root_token_mint,
                &TokenPricingSource::Mock {
                    numerator: vec![MockAsset::SOL(1_234_567_890 * 2)],
                    denominator: 1_234_567_890,
                },
            )
            .unwrap();

        let pegged_token_mint = Pubkey::new_unique();
        pricing_service
            .resolve_token_pricing_source(
                &pegged_token_mint,
                &TokenPricingSource::PeggedToken {
                    address: root_token_mint,
                },
            )
            .unwrap();

        let mut root_token_value = TokenValue::default();
        pricing_service
            .flatten_token_value(&root_token_mint, &mut root_token_value, false)
            .unwrap();

        let mut pegged_token_vaule = TokenValue::default();
        pricing_service
            .flatten_token_value(&pegged_token_mint, &mut pegged_token_vaule, false)
            .unwrap();

        assert_eq!(root_token_value, pegged_token_vaule);

        let basket_token_mint = Pubkey::new_unique();
        pricing_service
            .resolve_token_pricing_source(
                &basket_token_mint,
                &TokenPricingSource::Mock {
                    numerator: vec![
                        MockAsset::Token(root_token_mint, 50),
                        MockAsset::Token(pegged_token_mint, 50),
                        MockAsset::SOL(0),
                    ],
                    denominator: 100,
                },
            )
            .unwrap();

        let receipt_token_mint = Pubkey::new_unique();
        pricing_service
            .resolve_token_pricing_source(
                &receipt_token_mint,
                &TokenPricingSource::Mock {
                    numerator: vec![
                        MockAsset::Token(root_token_mint, 100),
                        MockAsset::Token(pegged_token_mint, 200),
                        MockAsset::Token(basket_token_mint, 100),
                        MockAsset::SOL(0),
                    ],
                    denominator: 400,
                },
            )
            .unwrap();

        let mut receipt_token_value = TokenValue::default();
        pricing_service
            .flatten_token_value(&receipt_token_mint, &mut receipt_token_value, false)
            .unwrap();
        assert_eq!(
            receipt_token_value,
            TokenValue {
                numerator: vec![
                    Asset::Token(
                        root_token_mint,
                        pricing_service
                            .get_token_pricing_source(&root_token_mint)
                            .cloned(),
                        150
                    ),
                    Asset::Token(
                        pegged_token_mint,
                        pricing_service
                            .get_token_pricing_source(&pegged_token_mint)
                            .cloned(),
                        250
                    ),
                ],
                denominator: 400,
            }
        );

        pricing_service
            .flatten_token_value(&receipt_token_mint, &mut receipt_token_value, true)
            .unwrap();
        assert_eq!(
            receipt_token_value,
            TokenValue {
                numerator: vec![Asset::Token(
                    root_token_mint,
                    pricing_service
                        .get_token_pricing_source(&root_token_mint)
                        .cloned(),
                    400
                )],
                denominator: 400
            }
        );

        let root_exchange_rate = pricing_service
            .get_asset_exchange_ratio(Some(&root_token_mint), &receipt_token_mint)
            .unwrap();
        assert_eq!(root_exchange_rate.unwrap(), (400, 400));

        let pegged_exchange_rate = pricing_service
            .get_asset_exchange_ratio(Some(&pegged_token_mint), &receipt_token_mint)
            .unwrap();
        assert_eq!(pegged_exchange_rate.unwrap(), (400, 400));

        let pegged_exchange_rate_rev = pricing_service
            .get_asset_exchange_ratio(Some(&receipt_token_mint), &pegged_token_mint)
            .unwrap();
        assert_eq!(pegged_exchange_rate_rev.unwrap(), (400, 400));

        let basket_exchange_rate = pricing_service
            .get_asset_exchange_ratio(Some(&basket_token_mint), &receipt_token_mint)
            .unwrap();
        assert_eq!(basket_exchange_rate.unwrap(), (2000000000, 2000000000));

        let sol_exchange_rate = pricing_service
            .get_asset_exchange_ratio(None, &receipt_token_mint)
            .unwrap();
        assert_eq!(sol_exchange_rate.unwrap(), (1000000000, 2000000000));
    }

    #[test]
    fn test_resolve_token_pricing_source() {
        let mut pricing_service = PricingService::new(&[]);

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
        let mut pricing_service = PricingService::new(&[]);

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
            .flatten_token_value(&basket_mint_49_10, &mut token_value_as_atomic, false)
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
            .flatten_token_value(&basket_mint_88_10, &mut token_value_as_atomic, false)
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
