use crate::errors::ErrorCode;
use crate::modules::fund::FundReceiptTokenValueProvider;
use crate::modules::normalization::NormalizedTokenPoolValueProvider;
use crate::modules::pricing::{
    Asset, MockPricingSourceValueProvider, TokenPricingSource, TokenValue, TokenValueProvider,
};
use crate::modules::staking::{MarinadeStakePoolValueProvider, SPLStakePoolValueProvider};
use crate::utils;
use anchor_lang::prelude::*;
use anchor_spl::token::accessor::{amount, mint};
use anchor_spl::token_interface::Mint;
use std::collections::BTreeMap;

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

    pub fn register_token_pricing_source(
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
                SPLStakePoolValueProvider::resolve_underlying_assets(
                    token_pricing_source,
                    vec![account1],
                )?
            }
            TokenPricingSource::MarinadeStakePool { address } => {
                let account1 = self
                    .token_pricing_source_accounts_map
                    .get(address)
                    .ok_or_else(|| error!(ErrorCode::TokenPricingSourceAccountNotFoundException))?;
                MarinadeStakePoolValueProvider::resolve_underlying_assets(
                    token_pricing_source,
                    vec![account1],
                )?
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
                NormalizedTokenPoolValueProvider::resolve_underlying_assets(
                    token_pricing_source,
                    vec![account1, account2],
                )?
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
                FundReceiptTokenValueProvider::resolve_underlying_assets(
                    token_pricing_source,
                    vec![account1, account2],
                )?
            }
            #[cfg(test)]
            TokenPricingSource::Mock { .. } => {
                MockPricingSourceValueProvider::resolve_underlying_assets(
                    token_pricing_source,
                    vec![],
                )?
            }
        };

        // expand supported tokens recursively
        token_value.numerator.iter().try_for_each(|asset| {
            if let Asset::TOKEN(token_mint, Some(token_pricing_source), _) = asset {
                self.register_token_pricing_source(token_mint, token_pricing_source)?;
            }
            Ok::<(), Error>(())
        })?;

        // check already registered token
        if self.token_value_map.contains_key(token_mint) {
            require_eq!(self.token_value_map.get(token_mint).unwrap(), &token_value);
            return Ok(());
        }

        // store resolved token value
        self.token_value_map.insert(*token_mint, token_value);
        Ok(())
    }

    // returns (total sol value of the token, total token amount)
    fn resolve_token_total_value_as_sol(&self, token_mint: &Pubkey) -> Result<(u64, u64)> {
        let token_value = self
            .token_value_map
            .get(token_mint)
            .ok_or_else(|| error!(ErrorCode::TokenPricingSourceAccountNotFoundException))?;
        let mut total_sol_amount = 0;

        for asset in &token_value.numerator {
            match asset {
                Asset::SOL(sol_amount) => {
                    total_sol_amount += sol_amount;
                }
                Asset::TOKEN(nested_token_mint, _, nested_token_amount) => {
                    let nested_sol_amount =
                        self.get_token_amount_as_sol(nested_token_mint, *nested_token_amount)?;
                    total_sol_amount += nested_sol_amount;
                }
            }
        }

        Ok((total_sol_amount, token_value.denominator))
    }

    pub fn get_sol_amount_as_token(&self, token_mint: &Pubkey, sol_amount: u64) -> Result<u64> {
        let (total_token_value_as_sol, total_token_amount) =
            self.resolve_token_total_value_as_sol(token_mint)?;
        utils::get_proportional_amount(sol_amount, total_token_amount, total_token_value_as_sol)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }

    pub fn get_token_amount_as_sol(&self, token_mint: &Pubkey, token_amount: u64) -> Result<u64> {
        let (total_token_value_as_sol, total_token_amount) =
            self.resolve_token_total_value_as_sol(token_mint)?;
        utils::get_proportional_amount(token_amount, total_token_value_as_sol, total_token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }
}
