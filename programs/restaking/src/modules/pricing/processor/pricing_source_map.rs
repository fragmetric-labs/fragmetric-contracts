use std::collections::BTreeMap;

use anchor_lang::prelude::*;

use crate::{errors::ErrorCode, modules::pricing::*};

pub(in crate::modules) fn find_related_pricing_source_accounts<'info>(
    pricing_source: &TokenPricingSource,
    // pricing_source_accounts: &'info [AccountInfo<'info>],
    pubkey_to_account_map: &BTreeMap<Pubkey, &'info AccountInfo<'info>>,
) -> Result<Vec<&'info AccountInfo<'info>>> {
    match pricing_source {
        TokenPricingSource::SPLStakePool { address }
        | TokenPricingSource::MarinadeStakePool { address } => Ok(vec![*pubkey_to_account_map
            .get(address)
            .ok_or_else(|| error!(ErrorCode::FundTokenPricingSourceNotFoundException))?]),
        TokenPricingSource::NormalizedTokenPool { mint, config } => {
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
