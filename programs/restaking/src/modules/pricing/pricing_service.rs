use anchor_lang::prelude::*;
use bytemuck::Zeroable;
use once_cell::unsync::OnceCell;
use primitive_types::U256;

use crate::errors::ErrorCode;
use crate::modules::fund::FundReceiptTokenValueProvider;
use crate::modules::normalization::NormalizedTokenPoolValueProvider;
use crate::modules::restaking::{JitoRestakingVaultValueProvider, SolvBTCVaultValueProvider};
use crate::modules::staking::{
    MarinadeStakePoolValueProvider, SPLStakePool, SPLStakePoolValueProvider,
    SanctumMultiValidatorSPLStakePool, SanctumSingleValidatorSPLStakePool,
};
use crate::modules::swap::OrcaDEXLiquidityPoolValueProvider;

use super::*;

const PRICING_SERVICE_EXPECTED_TOKENS_SIZE: usize = 34; // MAX=34 (ST=16 + NT=1 + VRT=16 + RT=1)

pub(in crate::modules) struct PricingService<'info> {
    token_pricing_sources_account_infos: Vec<&'info AccountInfo<'info>>,
    token_mints: Vec<Pubkey>,
    token_pricing_sources: Vec<TokenPricingSource>,
    token_values: Vec<TokenValue>,
    token_value_as_mirco_lamports: Vec<OnceCell<(u128, u128)>>,
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
            token_value_as_mirco_lamports: Vec::with_capacity(PRICING_SERVICE_EXPECTED_TOKENS_SIZE),
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

            // clear cache
            self.token_value_as_mirco_lamports[index].take();

            index
        } else {
            self.token_mints.push(*token_mint);
            self.token_pricing_sources
                .push(token_pricing_source.clone());
            // First we just push dummy TokenValue, and it will be updated soon!!
            self.token_values.push(TokenValue::default());
            self.token_value_as_mirco_lamports.push(OnceCell::new());
            self.token_mints.len() - 1
        };
        *updated_token_values_index_bitmap |= 1 << token_index;

        // resolve underlying assets for each pricing source' value provider adapter
        match token_pricing_source {
            TokenPricingSource::SPLStakePool { address } => {
                let pricing_source_accounts =
                    [self.get_token_pricing_source_account_info(address)?];
                SPLStakePoolValueProvider::<SPLStakePool>::new().resolve_underlying_assets(
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
                SPLStakePoolValueProvider::<SanctumSingleValidatorSPLStakePool>::new()
                    .resolve_underlying_assets(
                        token_mint,
                        &pricing_source_accounts,
                        &mut self.token_values[token_index],
                    )?
            }
            TokenPricingSource::PeggedToken { address } => {
                require_keys_neq!(*address, *token_mint);
                self.token_values[token_index] = self.get_token_value(address)?.clone();
            }
            TokenPricingSource::SolvBTCVault { address } => {
                let pricing_source_accounts =
                    [self.get_token_pricing_source_account_info(address)?];
                SolvBTCVaultValueProvider.resolve_underlying_assets(
                    token_mint,
                    &pricing_source_accounts,
                    &mut self.token_values[token_index],
                )?
            }
            TokenPricingSource::SanctumMultiValidatorSPLStakePool { address } => {
                let pricing_source_accounts =
                    [self.get_token_pricing_source_account_info(address)?];
                SPLStakePoolValueProvider::<SanctumMultiValidatorSPLStakePool>::new()
                    .resolve_underlying_assets(
                        token_mint,
                        &pricing_source_accounts,
                        &mut self.token_values[token_index],
                    )?
            }
            TokenPricingSource::VirtualVault { .. } => {
                self.token_values[token_index] = TokenValue::default();
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

    /// returns the token value in (numerator_as_micro_lamports, denominator_as_micro_token)
    fn get_token_value_as_mirco_lamports(&self, token_mint: &Pubkey) -> Result<(u128, u128)> {
        let token_index = self
            .get_token_index(token_mint)
            .ok_or_else(|| error!(ErrorCode::TokenPricingSourceAccountNotFoundError))?;

        self.token_value_as_mirco_lamports[token_index]
            .get_or_try_init(|| -> Result<(u128, u128)> {
                let token_value = self.get_token_value(token_mint)?;
                let mut micro_lamports = 0u128;

                for asset in &token_value.numerator {
                    match asset {
                        Asset::SOL(sol_amount) => {
                            micro_lamports += (*sol_amount as u128) * 1_000_000;
                        }
                        Asset::Token(nested_token_mint, _, nested_token_amount) => {
                            let (numerator_as_micro_lamports, denominator_as_micro_token) =
                                self.get_token_value_as_mirco_lamports(nested_token_mint)?;
                            micro_lamports += Self::get_proportional_amount_u128(
                                (*nested_token_amount as u128) * 1_000_000,
                                numerator_as_micro_lamports,
                                denominator_as_micro_token,
                            )?;
                        }
                    }
                }

                Ok((
                    micro_lamports,
                    (token_value.denominator as u128) * 1_000_000,
                ))
            })
            .cloned()
    }

    /// Convert `from_asset_amount` (in the unit of `from_asset_mint` or SOL) into the equivalent amount of
    /// `to_asset_mint` (or SOL). To avoid cumulative flooring drift during repeated deposits/withdrawals,
    /// It can optionally process a mutable `to_asset_residual_micro_amount` slot. Any fractional "dust"
    /// below the asset’s smallest unit will be stored there (in micro-units) and reapplied on the next call.
    fn get_asset_amount_as_asset(
        &self,
        from_asset_mint: Option<&Pubkey>,
        from_asset_amount: u64,
        to_asset_mint: Option<&Pubkey>,
        to_asset_residual_micro_amount: Option<&mut u64>,
    ) -> Result<u64> {
        match from_asset_mint {
            None => {
                match to_asset_mint {
                    None => {
                        // return error instead of returning back given lamports as assuming the caller tries invalid conversion
                        err!(ErrorCode::CalculationArithmeticException)
                    }
                    Some(to_token_mint) => {
                        let from_micro_lamports = (from_asset_amount as u128) * 1_000_000;
                        let (to_numerator_as_micro_lamports, to_denominator_as_micro_token) =
                            self.get_token_value_as_mirco_lamports(to_token_mint)?;
                        let mut to_micro_token = Self::get_proportional_amount_u128(
                            from_micro_lamports,
                            to_denominator_as_micro_token,
                            to_numerator_as_micro_lamports,
                        )?;

                        if let Some(to_token_residual_micro_amount) = to_asset_residual_micro_amount
                        {
                            to_micro_token += *to_token_residual_micro_amount as u128;

                            let new_to_token_residual_micro_amount =
                                u64::try_from(to_micro_token % 1_000_000)?;
                            *to_token_residual_micro_amount = new_to_token_residual_micro_amount;
                        };

                        let to_token = u64::try_from(to_micro_token / 1_000_000)?;
                        Ok(to_token)
                    }
                }
            }
            Some(from_token_mint) => {
                let (from_numerator_as_micro_lamports, from_denominator_as_micro_token) =
                    self.get_token_value_as_mirco_lamports(from_token_mint)?;

                match to_asset_mint {
                    None => {
                        let from_micro_token = (from_asset_amount as u128) * 1_000_000;
                        let mut to_micro_lamports = Self::get_proportional_amount_u128(
                            from_micro_token,
                            from_numerator_as_micro_lamports,
                            from_denominator_as_micro_token,
                        )?;

                        if let Some(to_lamports_residual_micro_amount) =
                            to_asset_residual_micro_amount
                        {
                            to_micro_lamports += *to_lamports_residual_micro_amount as u128;

                            let new_to_lamports_residual_micro_amount =
                                u64::try_from(to_micro_lamports % 1_000_000)?;
                            *to_lamports_residual_micro_amount =
                                new_to_lamports_residual_micro_amount;
                        }

                        let to_lamports = u64::try_from(to_micro_lamports / 1_000_000)?;
                        Ok(to_lamports)
                    }
                    Some(to_token_mint) => {
                        if to_asset_mint == from_asset_mint {
                            // return error as assuming the caller tries invalid conversion
                            return err!(ErrorCode::CalculationArithmeticException);
                        }

                        let (to_numerator_as_micro_lamports, to_denominator_as_micro_token) =
                            self.get_token_value_as_mirco_lamports(to_token_mint)?;

                        let micro_micro_numerator = U256::from(from_numerator_as_micro_lamports)
                            .checked_mul(U256::from(to_denominator_as_micro_token))
                            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

                        let micro_micro_denominator = U256::from(from_denominator_as_micro_token)
                            .checked_mul(U256::from(to_numerator_as_micro_lamports))
                            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

                        if micro_micro_numerator == micro_micro_denominator
                            || micro_micro_denominator.is_zero() && from_asset_amount == 0
                        {
                            return Ok(from_asset_amount);
                        }

                        let mut to_micro_token =
                            U256::from((from_asset_amount as u128) * 1_000_000)
                                .checked_mul(micro_micro_numerator)
                                .and_then(|v| v.checked_div(micro_micro_denominator))
                                .and_then(|v| u128::try_from(v).ok())
                                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

                        if let Some(to_token_residual_micro_amount) = to_asset_residual_micro_amount
                        {
                            to_micro_token += *to_token_residual_micro_amount as u128;

                            let new_to_token_residual_micro_amount =
                                u64::try_from(to_micro_token % 1_000_000)?;
                            *to_token_residual_micro_amount = new_to_token_residual_micro_amount;
                        };

                        let to_token = u64::try_from(to_micro_token / 1_000_000)?;
                        Ok(to_token)
                    }
                }
            }
        }
    }

    #[inline(always)]
    pub fn convert_asset_amount(
        &self,
        from_asset_mint: Option<&Pubkey>,
        from_asset_amount: u64,
        to_asset_mint: Option<&Pubkey>,
        to_asset_residual_micro_amount: &mut u64,
    ) -> Result<u64> {
        self.get_asset_amount_as_asset(
            from_asset_mint,
            from_asset_amount,
            to_asset_mint,
            Some(to_asset_residual_micro_amount),
        )
    }

    #[inline(always)]
    pub fn get_sol_amount_as_token(&self, to_token_mint: &Pubkey, sol_amount: u64) -> Result<u64> {
        self.get_asset_amount_as_asset(None, sol_amount, Some(to_token_mint), None)
    }

    #[inline(always)]
    pub fn get_token_amount_as_sol(
        &self,
        from_token_mint: &Pubkey,
        token_amount: u64,
    ) -> Result<u64> {
        self.get_asset_amount_as_asset(Some(from_token_mint), token_amount, None, None)
    }

    #[inline(always)]
    pub fn get_token_amount_as_token(
        &self,
        from_asset_mint: &Pubkey,
        from_asset_amount: u64,
        to_token_mint: &Pubkey,
    ) -> Result<u64> {
        self.get_asset_amount_as_asset(
            Some(from_asset_mint),
            from_asset_amount,
            Some(to_token_mint),
            None,
        )
    }

    #[inline(always)]
    pub fn get_token_amount_as_asset(
        &self,
        from_asset_mint: &Pubkey,
        from_asset_amount: u64,
        to_asset_mint: Option<&Pubkey>,
    ) -> Result<u64> {
        self.get_asset_amount_as_asset(
            Some(from_asset_mint),
            from_asset_amount,
            to_asset_mint,
            None,
        )
    }

    /// This is for display or informational purposes only.
    pub fn get_one_token_amount_as_sol(
        &self,
        from_token_mint: &Pubkey,
        from_token_decimals: u8,
    ) -> Result<Option<u64>> {
        let (_, denominator_as_micro_token) =
            self.get_token_value_as_mirco_lamports(from_token_mint)?;
        Ok(if denominator_as_micro_token == 0 {
            None
        } else {
            Some({
                let token_amount = 10u64.pow(from_token_decimals as u32);

                self.get_asset_amount_as_asset(Some(from_token_mint), token_amount, None, None)?
            })
        })
    }

    /// This is for display or informational purposes only.
    pub fn get_one_token_amount_as_token(
        &self,
        from_token_mint: &Pubkey,
        from_token_decimals: u8,
        to_token_mint: &Pubkey,
    ) -> Result<Option<u64>> {
        let (_, denominator_as_micro_token_from) =
            self.get_token_value_as_mirco_lamports(from_token_mint)?;
        let (_, denominator_as_micro_token_to) =
            self.get_token_value_as_mirco_lamports(to_token_mint)?;
        Ok(
            if denominator_as_micro_token_from == 0 || denominator_as_micro_token_to == 0 {
                None
            } else {
                Some({
                    let token_amount = 10u64.pow(from_token_decimals as u32);

                    self.get_asset_amount_as_asset(
                        Some(from_token_mint),
                        token_amount,
                        Some(to_token_mint),
                        None,
                    )?
                })
            },
        )
    }

    /// This is for display or informational purposes only.
    /// Computes a flattened breakdown of the given token’s value into its underlying assets.
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
            result.numerator.reserve_exact(token_value.numerator.len());
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

        Ok(())
    }

    pub fn flatten_token_value_pod(
        &self,
        token_mint: &Pubkey,
        result: &mut TokenValuePod,
    ) -> Result<()> {
        let token_value = self.get_token_value(token_mint)?;

        *result = TokenValuePod::zeroed();
        result.denominator = token_value.denominator;

        // If token value is atomic, then it is already summarized.
        if token_value.is_atomic() {
            result.num_numerator = token_value.numerator.len() as u64;
            for (index, asset) in token_value.numerator.iter().enumerate() {
                asset.serialize_as_pod(&mut result.numerator[index]);
            }
        } else {
            for asset in &token_value.numerator {
                match asset {
                    Asset::SOL(sol_amount) => {
                        self.flatten_token_value_pod_of_sol(*sol_amount, result);
                    }
                    Asset::Token(token_mint, token_pricing_source, token_amount) => {
                        self.flatten_token_value_pod_of_token(
                            token_mint,
                            token_pricing_source.as_ref(),
                            *token_amount,
                            result,
                        )?;
                    }
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

    #[inline(always)]
    fn flatten_token_value_pod_of_sol(&self, sol_amount: u64, result: &mut TokenValuePod) {
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
                    let nested_sol_amount = Self::get_proportional_amount_u64(
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
                    let nested_token_amount = Self::get_proportional_amount_u64(
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

    fn flatten_token_value_pod_of_token(
        &self,
        token_mint: &Pubkey,
        token_pricing_source: Option<&TokenPricingSource>,
        token_amount: u64,
        result: &mut TokenValuePod,
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
                    let nested_sol_amount = Self::get_proportional_amount_u64(
                        *nested_sol_amount,
                        token_amount,
                        token_value.denominator,
                    )?;
                    self.flatten_token_value_pod_of_sol(nested_sol_amount, result);
                }
                Asset::Token(
                    nested_token_mint,
                    nested_token_pricing_source,
                    nested_token_amount,
                ) => {
                    let nested_token_amount = Self::get_proportional_amount_u64(
                        *nested_token_amount,
                        token_amount,
                        token_value.denominator,
                    )?;
                    self.flatten_token_value_pod_of_token(
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

    /// This is for precise calculation.
    fn get_proportional_amount_u128(
        amount: u128,
        numerator: u128,
        denominator: u128,
    ) -> Result<u128> {
        if numerator == denominator || denominator == 0 && amount == 0 {
            return Ok(amount);
        }
        if amount == denominator {
            return Ok(numerator);
        }

        U256::from(amount)
            .checked_mul(U256::from(numerator))
            .and_then(|numerator| numerator.checked_div(U256::from(denominator)))
            .and_then(|amount| u128::try_from(amount).ok())
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }

    /// This is for display or informational purposes only.
    fn get_proportional_amount_u64(amount: u64, numerator: u64, denominator: u64) -> Result<u64> {
        if numerator == denominator || denominator == 0 && amount == 0 {
            return Ok(amount);
        }
        if amount == denominator {
            return Ok(numerator);
        }

        u64::try_from(amount as u128 * numerator as u128 / denominator as u128)
            .map_err(|_| error!(ErrorCode::CalculationArithmeticException))
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

        let stable_conversion = pricing_service
            .get_token_amount_as_token(&token_mint, 1_000_000_000, &stable_fund_mint)
            .unwrap();
        assert_eq!(stable_conversion, 1_000_000_000);

        let stable_conversion_rev = pricing_service
            .get_token_amount_as_token(&stable_fund_mint, 1_000_000_000, &token_mint)
            .unwrap();
        assert_eq!(stable_conversion_rev, 1_000_000_000);

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

        let increased_conversion = pricing_service
            .get_token_amount_as_token(&token_mint, 1_000_000_000, &increased_fund_mint)
            .unwrap();
        assert_eq!(increased_conversion, 1_000_000_000 / 2);
        assert_eq!(
            pricing_service
                .get_token_amount_as_token(&token_mint, 1234, &stable_fund_mint)
                .unwrap(),
            1234
        );
        assert_eq!(
            pricing_service
                .get_sol_amount_as_token(&stable_fund_mint, 1234)
                .unwrap(),
            617
        );

        let increased_conversion_rev = pricing_service
            .get_token_amount_as_token(&increased_fund_mint, 1_000_000_000, &token_mint)
            .unwrap();
        assert_eq!(increased_conversion_rev, 1_000_000_000 * 2);
        assert_eq!(
            pricing_service
                .get_token_amount_as_token(&token_mint, 1234, &increased_fund_mint)
                .unwrap(),
            617
        );
        assert_eq!(
            pricing_service
                .get_sol_amount_as_token(&increased_fund_mint, 1234)
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
            .flatten_token_value(&root_token_mint, &mut root_token_value)
            .unwrap();

        let mut pegged_token_vaule = TokenValue::default();
        pricing_service
            .flatten_token_value(&pegged_token_mint, &mut pegged_token_vaule)
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
            .flatten_token_value(&receipt_token_mint, &mut receipt_token_value)
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

        let root_conversion = pricing_service
            .get_token_amount_as_token(&root_token_mint, 1_000_000_000, &receipt_token_mint)
            .unwrap();
        assert_eq!(root_conversion, 1_000_000_000);

        let pegged_conversion = pricing_service
            .get_token_amount_as_token(&pegged_token_mint, 1_000_000_000, &receipt_token_mint)
            .unwrap();
        assert_eq!(pegged_conversion, 1_000_000_000);

        let pegged_conversion_rev = pricing_service
            .get_token_amount_as_token(&receipt_token_mint, 1_000_000_000, &pegged_token_mint)
            .unwrap();
        assert_eq!(pegged_conversion_rev, 1_000_000_000);

        let basket_conversion = pricing_service
            .get_token_amount_as_token(&basket_token_mint, 1_000_000_000, &receipt_token_mint)
            .unwrap();
        assert_eq!(basket_conversion, 1_000_000_000);

        let sol_conversion = pricing_service
            .get_sol_amount_as_token(&receipt_token_mint, 1_000_000_000)
            .unwrap();
        assert_eq!(sol_conversion, 1_000_000_000 / 2);
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
            .flatten_token_value(&basket_mint_49_10, &mut token_value_as_atomic)
            .unwrap();
        let token_value_as_micro_lamports = pricing_service
            .get_token_value_as_mirco_lamports(&basket_mint_49_10)
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

        assert_eq!(token_value_as_micro_lamports.0, 49_000_000);
        assert_eq!(token_value_as_micro_lamports.1, 10_000_000);
        assert_eq!(
            (token_value_as_atomic.denominator as u128) * 1_000_000,
            token_value_as_micro_lamports.1
        );
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
        let token_value_as_micro_lamports = pricing_service
            .get_token_value_as_mirco_lamports(&basket_mint_88_10)
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

        assert_eq!(token_value_as_micro_lamports.0, 88_000_000);
        assert_eq!(token_value_as_micro_lamports.1, 10_000_000);
        assert_eq!(
            (token_value_as_atomic.denominator as u128) * 1_000_000,
            token_value_as_micro_lamports.1
        );
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
