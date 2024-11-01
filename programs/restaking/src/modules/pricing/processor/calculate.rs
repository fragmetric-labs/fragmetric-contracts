use anchor_lang::prelude::*;

use crate::modules::pricing::*;

#[inline(always)]
pub(in crate::modules) fn calculate_token_amount_as_sol(
    mint: Pubkey,
    pricing_source_map: &TokenPricingSourceMap,
    token_amount: u64,
) -> Result<u64> {
    create_token_amount_as_sol_calculator(mint, pricing_source_map)?
        .calculate_token_amount_as_sol(token_amount, pricing_source_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_pricing_source() {
        let mock_mint = Pubkey::new_unique();
        let pricing_source_map = [(mock_mint, (TokenPricingSource::Mock, vec![]))].into();
        let token_amount =
            calculate_token_amount_as_sol(mock_mint, &pricing_source_map, 10000).unwrap();
        assert_eq!(token_amount, 12000);
    }
}
