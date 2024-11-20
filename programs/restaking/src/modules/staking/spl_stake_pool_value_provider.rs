use anchor_lang::prelude::*;

use crate::modules::pricing::{Asset, TokenValue, TokenValueProvider};

use super::SPLStakePoolService;

pub struct SPLStakePoolValueProvider;

impl TokenValueProvider for SPLStakePoolValueProvider {
    fn resolve_underlying_assets<'a, 'info: 'a>(
        self,
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
