use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::pricing::*;

pub(in crate::modules) fn calculate_token_value<'info>(
    accounts: &'info [AccountInfo<'info>],
    source: &TokenPricingSource,
    amount: u64,
) -> Result<TokenValue> {
    token_price_calculator(source, accounts)?.calculate_token_value(amount)
}

fn token_price_calculator<'info>(
    source: &TokenPricingSource,
    accounts: &'info [AccountInfo<'info>],
) -> Result<Box<dyn TokenValueCalculator>> {
    match source {
        TokenPricingSource::SPLStakePool { address } => {
            let account = find_token_pricing_source_by_key(accounts, address)?;
            Ok(Box::new(
                Account::<SplStakePool>::try_from(account)?.into_inner(),
            ))
        }
        TokenPricingSource::MarinadeStakePool { address } => {
            let account = find_token_pricing_source_by_key(accounts, address)?;
            Ok(Box::new(
                Account::<MarinadeStakePool>::try_from(account)?.into_inner(),
            ))
        }
        #[cfg(test)]
        TokenPricingSource::Mock => Ok(Box::new(MockPriceSource)),
    }
}

fn find_token_pricing_source_by_key<'info>(
    accounts: &'info [AccountInfo<'info>],
    key: &Pubkey,
) -> Result<&'info AccountInfo<'info>> {
    accounts
        .iter()
        .find(|account| account.key == key)
        .ok_or_else(|| error!(ErrorCode::FundTokenPricingSourceNotFoundException))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_price() {
        let pricing_source = TokenPricingSource::Mock;
        let token_value = calculate_token_value(&[], &pricing_source, 10000).unwrap();
        assert!(matches!(token_value, TokenValue::SOL(12000)));
    }
}
