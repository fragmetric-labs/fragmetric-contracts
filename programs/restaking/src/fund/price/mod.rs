use anchor_lang::prelude::*;

use crate::{error::ErrorCode, fund::*};

use source::*;

mod source;

impl TokenInfo {
    pub(super) fn calculate_sol_from_tokens(&self, token_amount: u64) -> Result<u64> {
        crate::utils::proportional_amount(
            token_amount,
            self.token_to_sol_value,
            self.token_amount_in,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationFailure))
    }
}

impl Fund {
    pub(crate) fn update_token_prices<'info>(
        &mut self,
        sources: &[&AccountInfo<'info>],
    ) -> Result<()> {
        for token in &mut self.supported_tokens {
            let calculator: Box<dyn TokenPriceCalculator> = match &token.pricing_source {
                PricingSource::SPLStakePool { address } => {
                    let account = find_pricing_source_by_key(sources, address)?;
                    let spl_stake_pool =
                        ToCalculator::<SplStakePool>::to_calculator_checked(account)?;
                    Box::new(spl_stake_pool)
                }
                PricingSource::MarinadeStakePool { address } => {
                    let account = find_pricing_source_by_key(sources, address)?;
                    let marinade_stake_pool =
                        ToCalculator::<MarinadeStakePool>::to_calculator_checked(account)?;
                    Box::new(marinade_stake_pool)
                }
            };
            token.token_to_sol_value = calculator.calculate_token_price(token.token_amount_in)?;
        }

        Ok(())
    }

    pub(super) fn receipt_token_price(
        &self,
        receipt_token_decimal: u8,
        receipt_token_total_supply: u64,
    ) -> Result<u64> {
        self.calculate_sol_from_receipt_tokens(
            10u64
                .checked_pow(receipt_token_decimal as u32)
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
            .fold(Ok(self.sol_amount_in), |sum, token| {
                sum.and_then(|sum| {
                    sum.checked_add(token.token_to_sol_value)
                        .ok_or_else(|| error!(ErrorCode::CalculationFailure))
                })
            })
    }
}

fn find_pricing_source_by_key<'a, 'info: 'a>(
    sources: &[&'a AccountInfo<'info>],
    key: &Pubkey,
) -> Result<&'a AccountInfo<'info>> {
    Ok(sources
        .iter()
        .find(|account| account.key == key)
        .ok_or_else(|| error!(ErrorCode::FundPricingSourceNotFound))?)
}
