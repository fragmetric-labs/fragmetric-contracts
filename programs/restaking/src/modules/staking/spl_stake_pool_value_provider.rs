use crate::modules::pricing::{Asset, TokenPricingSource, TokenValue, TokenValueProvider};
use crate::utils;
use anchor_lang::prelude::*;
use spl_stake_pool::state::StakePool as SPLStakePoolAccount;

pub struct SPLStakePoolValueProvider;

impl TokenValueProvider for SPLStakePoolValueProvider {
    fn resolve_underlying_assets(
        _token_pricing_source: &TokenPricingSource,
        pricing_source_accounts: Vec<&AccountInfo>,
    ) -> Result<TokenValue> {
        require_eq!(pricing_source_accounts.len(), 1);

        let pool_account =
            SPLStakePoolAccount::deserialize(&mut &**pricing_source_accounts[0].try_borrow_data()?)
                .map_err(|_| error!(ErrorCode::AccountDidNotDeserialize))?;
        require_eq!(pool_account.is_valid(), true);

        Ok(TokenValue {
            numerator: vec![Asset::SOL(pool_account.total_lamports)],
            denominator: pool_account.pool_token_supply,
        })
    }
}
