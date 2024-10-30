use anchor_lang::prelude::*;

use crate::errors::ErrorCode;

use super::*;

/// Price: 1 denominated unit = 1.2 lamports
pub struct MockPriceSource;

impl TokenAmountAsSOLCalculator for MockPriceSource {
    fn calculate_token_amount_as_sol(
        &self,
        token_amount: u64,
        _pricing_source_map: &TokenPricingSourceMap,
    ) -> Result<u64> {
        token_amount
            .checked_mul(6)
            .and_then(|amount| amount.checked_div(5))
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }
}
