use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{
    errors::ErrorCode,
    modules::{
        normalize::{self, NormalizedTokenPoolAccount},
        pricing::calculate_token_amount_as_sol,
    },
};

use super::*;

pub struct NormalizedTokenAmountAsSOLCalculator<'info> {
    normalized_token_pool_account: Account<'info, NormalizedTokenPoolAccount>,
    normalized_token_mint: InterfaceAccount<'info, Mint>,
}

impl<'info> TokenAmountAsSOLCalculator for NormalizedTokenAmountAsSOLCalculator<'info> {
    fn calculate_token_amount_as_sol(
        &self,
        token_amount: u64,
        pricing_source_map: &TokenPricingSourceMap,
    ) -> Result<u64> {
        let mut assets_total_amount_as_sol = 0u64;
        for (mint, token_amount) in self
            .normalized_token_pool_account
            .get_supported_tokens_locked_amount()
        {
            assets_total_amount_as_sol = assets_total_amount_as_sol
                .checked_add(calculate_token_amount_as_sol(
                    mint,
                    pricing_source_map,
                    token_amount,
                )?)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        }

        crate::utils::get_proportional_amount(
            token_amount,
            assets_total_amount_as_sol,
            self.normalized_token_mint.supply,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }
}

impl<'info> NormalizedTokenAmountAsSOLCalculator<'info> {
    pub(super) fn new(
        normalized_token_pool_account: Account<'info, NormalizedTokenPoolAccount>,
        normalized_token_mint: InterfaceAccount<'info, Mint>,
    ) -> Self {
        Self {
            normalized_token_pool_account,
            normalized_token_mint,
        }
    }
}
