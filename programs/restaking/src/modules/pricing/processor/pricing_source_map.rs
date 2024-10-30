use std::collections::BTreeMap;

use anchor_lang::prelude::*;

use crate::{errors::ErrorCode, modules::pricing::*};

pub(in crate::modules) fn create_pricing_source_map<'info>(
    mints_and_pricing_sources: Vec<(Pubkey, TokenPricingSource)>,
    pricing_source_accounts: &'info [AccountInfo<'info>],
) -> Result<TokenPricingSourceMap<'info>> {
    let mut pricing_source_map = TokenPricingSourceMap::new();
    let pubkey_to_account_map = pricing_source_accounts
        .iter()
        .map(|account| (account.key(), account))
        .collect();

    for (mint, pricing_source) in mints_and_pricing_sources {
        let accounts =
            find_related_pricing_source_accounts(&pricing_source, &pubkey_to_account_map)?;
        pricing_source_map.insert(mint, (pricing_source, accounts));
    }

    Ok(pricing_source_map)
}

fn find_related_pricing_source_accounts<'info>(
    pricing_source: &TokenPricingSource,
    // pricing_source_accounts: &'info [AccountInfo<'info>],
    pubkey_to_account_map: &BTreeMap<Pubkey, &'info AccountInfo<'info>>,
) -> Result<Vec<&'info AccountInfo<'info>>> {
    match pricing_source {
        TokenPricingSource::SPLStakePool { address }
        | TokenPricingSource::MarinadeStakePool { address } => Ok(vec![*pubkey_to_account_map
            .get(address)
            .ok_or_else(|| error!(ErrorCode::FundTokenPricingSourceNotFoundException))?]),
        TokenPricingSource::NormalizedTokenPool { mint_address: mint, pool_address: config } => {
            let mint = *pubkey_to_account_map
                .get(mint)
                .ok_or_else(|| error!(ErrorCode::FundTokenPricingSourceNotFoundException))?;
            let config = *pubkey_to_account_map
                .get(config)
                .ok_or_else(|| error!(ErrorCode::FundTokenPricingSourceNotFoundException))?;
            Ok(vec![mint, config])
        }
        #[cfg(test)]
        TokenPricingSource::Mock => Ok(vec![]),
    }
}
