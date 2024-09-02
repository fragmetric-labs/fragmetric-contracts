pub(super) mod source;

use anchor_lang::prelude::*;

use source::*;
use crate::{errors::ErrorCode};
use crate::modules::fund::{FundAccount, SupportedTokenInfo, TokenPricingSource};

impl SupportedTokenInfo {
    /// Simply it returns 10^token_decimals.
    fn token_lamports_per_token(&self) -> Result<u64> {
        10u64
            .checked_pow(self.decimals as u32)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }

    pub fn calculate_sol_from_tokens(&self, token_amount: u64) -> Result<u64> {
        crate::utils::proportional_amount(
            token_amount,
            self.price,
            self.token_lamports_per_token()?,
        )
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }
}

impl FundAccount {
    pub fn update_token_prices(&mut self, sources: &[AccountInfo]) -> Result<()> {
        for token in &mut self.supported_tokens {
            let token_lamports_per_token = token.token_lamports_per_token()?;
            match &token.pricing_source {
                TokenPricingSource::SPLStakePool { address } => {
                    let account = find_token_pricing_source_by_key(sources, address)?;
                    let spl_stake_pool =
                        ToCalculator::<SplStakePool>::to_calculator_checked(account)?;
                    token.price = spl_stake_pool.calculate_token_price(token_lamports_per_token)?;
                }
                TokenPricingSource::MarinadeStakePool { address } => {
                    let account = find_token_pricing_source_by_key(sources, address)?;
                    let marinade_stake_pool =
                        ToCalculator::<MarinadeStakePool>::to_calculator_checked(account)?;
                    token.price =
                        marinade_stake_pool.calculate_token_price(token_lamports_per_token)?;
                }
            }
        }

        Ok(())
    }

    pub fn receipt_token_sol_value_per_token(
        &self,
        receipt_token_decimals: u8,
        receipt_token_total_supply: u64,
    ) -> Result<u64> {
        self.receipt_token_sol_value_for(
            10u64
                .checked_pow(receipt_token_decimals as u32)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?,
            receipt_token_total_supply,
        )
    }

    pub fn receipt_token_mint_amount_for(
        &self,
        sol_amount: u64,
        receipt_token_total_supply: u64,
    ) -> Result<u64> {
        crate::utils::proportional_amount(
            sol_amount,
            receipt_token_total_supply,
            self.assets_total_sol_value()?,
        )
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }

    pub fn receipt_token_sol_value_for(
        &self,
        receipt_token_amount: u64,
        receipt_token_total_supply: u64,
    ) -> Result<u64> {
        crate::utils::proportional_amount(
            receipt_token_amount,
            self.assets_total_sol_value()?,
            receipt_token_total_supply,
        )
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }

    pub fn assets_total_sol_value(&self) -> Result<u64> {
        // TODO: need to add the sum(operating sol/tokens) after supported_restaking_protocols add
        self.supported_tokens
            .iter()
            .try_fold(self.sol_operation_reserved_amount, |sum, token| {
                sum.checked_add(token.calculate_sol_from_tokens(token.operation_reserved_amount)?)
                    .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
            })
    }
}

fn find_token_pricing_source_by_key<'a, 'info: 'a>(
    sources: &'a [AccountInfo<'info>],
    key: &Pubkey,
) -> Result<&'a AccountInfo<'info>> {
    Ok(sources
        .iter()
        .find(|account| account.key == key)
        .ok_or_else(|| error!(ErrorCode::FundTokenPricingSourceNotFoundException))?)
}
