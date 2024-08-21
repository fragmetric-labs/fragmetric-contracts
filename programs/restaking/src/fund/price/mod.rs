use anchor_lang::prelude::*;

use crate::{error::ErrorCode, fund::*};

use source::*;

mod source;

impl TokenInfo {
    /// Simply it returns 10^token_decimals.
    fn token_lamports_per_token(&self) -> Result<u64> {
        10u64
            .checked_pow(self.token_decimals as u32)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))
    }

    pub(super) fn calculate_sol_from_tokens(&self, token_amount: u64) -> Result<u64> {
        crate::utils::proportional_amount(
            token_amount,
            self.token_price,
            self.token_lamports_per_token()?,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationFailure))
    }
}

impl Fund {
    pub(crate) fn update_token_prices(&mut self, sources: &[&AccountInfo]) -> Result<()> {
        for token in &mut self.supported_tokens {
            let token_lamports_per_token = token.token_lamports_per_token()?;
            match &token.token_pricing_source {
                TokenPricingSource::SPLStakePool { address } => {
                    let account = find_token_pricing_source_by_key(sources, address)?;
                    let spl_stake_pool =
                        ToCalculator::<SplStakePool>::to_calculator_checked(account)?;
                    token.token_price =
                        spl_stake_pool.calculate_token_price(token_lamports_per_token)?;
                }
                TokenPricingSource::MarinadeStakePool { address } => {
                    let account = find_token_pricing_source_by_key(sources, address)?;
                    let marinade_stake_pool =
                        ToCalculator::<MarinadeStakePool>::to_calculator_checked(account)?;
                    token.token_price =
                        marinade_stake_pool.calculate_token_price(token_lamports_per_token)?;
                }
            }
        }

        Ok(())
    }

    pub(super) fn receipt_token_price(
        &self,
        decimals: u8,
        receipt_token_total_supply: u64,
    ) -> Result<u64> {
        self.calculate_sol_from_receipt_tokens(
            10u64
                .checked_pow(decimals as u32)
                .ok_or_else(|| error!(ErrorCode::CalculationFailure))?,
            receipt_token_total_supply,
        )
    }

    pub(super) fn calculate_sol_from_receipt_tokens(
        &self,
        receipt_token_amount: u64,
        receipt_token_total_supply: u64,
    ) -> Result<u64> {
        crate::utils::proportional_amount(
            receipt_token_amount,
            self.total_sol_value()?,
            receipt_token_total_supply,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationFailure))
    }

    pub(super) fn calculate_receipt_tokens_from_sol(
        &self,
        sol_amount: u64,
        receipt_token_total_supply: u64,
    ) -> Result<u64> {
        crate::utils::proportional_amount(
            sol_amount,
            receipt_token_total_supply,
            self.total_sol_value()?,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationFailure))
    }

    pub(crate) fn total_sol_value(&self) -> Result<u64> {
        self.supported_tokens
            .iter()
            .try_fold(self.sol_amount_in, |sum, token| {
                sum.checked_add(token.calculate_sol_from_tokens(token.token_amount_in)?)
                    .ok_or_else(|| error!(ErrorCode::CalculationFailure))
            })
    }
}

fn find_token_pricing_source_by_key<'a, 'info: 'a>(
    sources: &[&'a AccountInfo<'info>],
    key: &Pubkey,
) -> Result<&'a AccountInfo<'info>> {
    Ok(sources
        .iter()
        .find(|account| account.key == key)
        .ok_or_else(|| error!(ErrorCode::FundTokenPricingSourceNotFound))?)
}
