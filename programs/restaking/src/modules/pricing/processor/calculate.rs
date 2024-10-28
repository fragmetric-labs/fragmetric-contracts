use std::collections::BTreeMap;

use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::pricing::*;

pub(in crate::modules) fn pricing_sources_map<'info>(
    accounts: &'info [AccountInfo<'info>],
) -> BTreeMap<Pubkey, &'info AccountInfo<'info>> {
    accounts
        .iter()
        .map(|account| (account.key(), account))
        .collect()
}

pub(in crate::modules) fn calculate_token_value<'info>(
    pricing_source_accounts: &BTreeMap<Pubkey, &'info AccountInfo<'info>>,
    source: &TokenPricingSource,
    amount: u64,
) -> Result<TokenValue> {
    token_value_calculator(pricing_source_accounts, source)?.calculate_token_value(amount)
}

fn token_value_calculator<'info>(
    pricing_source_accounts: &BTreeMap<Pubkey, &'info AccountInfo<'info>>,
    source: &TokenPricingSource,
) -> Result<Box<dyn TokenValueCalculator>> {
    match source {
        TokenPricingSource::SPLStakePool { address } => {
            let account = find_token_pricing_source_by_key(pricing_source_accounts, address)?;
            Ok(Box::new(
                Account::<SplStakePool>::try_from(account)?.into_inner(),
            ))
        }
        TokenPricingSource::MarinadeStakePool { address } => {
            let account = find_token_pricing_source_by_key(pricing_source_accounts, address)?;
            Ok(Box::new(
                Account::<MarinadeStakePool>::try_from(account)?.into_inner(),
            ))
        }
        #[cfg(test)]
        TokenPricingSource::Mock => Ok(Box::new(MockPriceSource)),
    }
}

fn find_token_pricing_source_by_key<'info>(
    pricing_source_accounts: &BTreeMap<Pubkey, &'info AccountInfo<'info>>,
    key: &Pubkey,
) -> Result<&'info AccountInfo<'info>> {
    Ok(*pricing_source_accounts
        .get(key)
        .ok_or_else(|| error!(ErrorCode::FundTokenPricingSourceNotFoundException))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_price() {
        let pricing_source = TokenPricingSource::Mock;
        let token_value =
            calculate_token_value(&Default::default(), &pricing_source, 10000).unwrap();
        assert!(matches!(token_value, TokenValue::SOL(12000)));
    }
}
