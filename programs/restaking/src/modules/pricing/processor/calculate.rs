use std::collections::BTreeMap;

use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::pricing::*;

pub(in crate::modules) fn create_pricing_sources_map<'info>(
    accounts: &'info [AccountInfo<'info>],
) -> BTreeMap<Pubkey, &'info AccountInfo<'info>> {
    accounts
        .iter()
        .map(|account| (account.key(), account))
        .collect()
}

#[inline(always)]
pub(in crate::modules) fn calculate_token_amount_as_sol<'info>(
    pricing_source_accounts: &BTreeMap<Pubkey, &'info AccountInfo<'info>>,
    source: &TokenPricingSource,
    amount: u64,
) -> Result<TokenAmount> {
    create_token_amount_as_sol_calculator(pricing_source_accounts, source)?
        .calculate_token_amount_as_sol(amount)
}

fn create_token_amount_as_sol_calculator<'info>(
    pricing_source_accounts: &BTreeMap<Pubkey, &'info AccountInfo<'info>>,
    source: &TokenPricingSource,
) -> Result<Box<dyn TokenAmountAsSOLCalculator>> {
    match source {
        TokenPricingSource::SPLStakePool { address } => Ok(Box::new(
            Account::<SplStakePool>::try_from(
                pricing_source_accounts
                    .get(address)
                    .ok_or_else(|| error!(ErrorCode::FundTokenPricingSourceNotFoundException))?,
            )?
            .into_inner(),
        )),
        TokenPricingSource::MarinadeStakePool { address } => Ok(Box::new(
            Account::<MarinadeStakePool>::try_from(
                pricing_source_accounts
                    .get(address)
                    .ok_or_else(|| error!(ErrorCode::FundTokenPricingSourceNotFoundException))?,
            )?
            .into_inner(),
        )),
        #[cfg(test)]
        TokenPricingSource::Mock => Ok(Box::new(MockPriceSource)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_pricing_source() {
        let pricing_source = TokenPricingSource::Mock;
        let token_amount =
            calculate_token_amount_as_sol(&Default::default(), &pricing_source, 10000).unwrap();
        assert!(matches!(token_amount, TokenAmount::SOLAmount(12000)));
    }
}
