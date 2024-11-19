use super::SPLStakePoolService;
use crate::modules::pricing::{Asset, TokenPricingSource, TokenValue, TokenValueProvider};
use crate::utils;
use anchor_lang::prelude::*;

pub struct SPLStakePoolValueProvider;

impl TokenValueProvider for SPLStakePoolValueProvider {
    fn resolve_underlying_assets<'a, 'info: 'a>(
        _token_pricing_source: &TokenPricingSource,
        pricing_source_accounts: Vec<&'a AccountInfo<'info>>,
    ) -> Result<TokenValue> {
        require_eq!(pricing_source_accounts.len(), 1);

        let pool_account =
            SPLStakePoolService::deserialize_pool_account(pricing_source_accounts[0])?;
        Ok(TokenValue {
            numerator: vec![Asset::SOL(pool_account.total_lamports)],
            denominator: pool_account.pool_token_supply,
        })
    }
}
