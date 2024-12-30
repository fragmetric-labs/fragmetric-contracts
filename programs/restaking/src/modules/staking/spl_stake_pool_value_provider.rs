use anchor_lang::prelude::*;

use crate::modules::{
    pricing::{Asset, TokenValue, TokenValueProvider},
    staking::SPLStakePool,
};

use super::SPLStakePoolService;

pub struct SPLStakePoolValueProvider;

impl TokenValueProvider for SPLStakePoolValueProvider {
    #[inline(never)]
    fn resolve_underlying_assets<'info>(
        self,
        token_mint: &Pubkey,
        pricing_source_accounts: &[&'info AccountInfo<'info>],
    ) -> Result<TokenValue> {
        require_eq!(pricing_source_accounts.len(), 1);

        let pool_account = SPLStakePoolService::<SPLStakePool>::deserialize_pool_account(
            pricing_source_accounts[0],
        )?;
        require_keys_eq!(pool_account.pool_mint, *token_mint);

        Ok(TokenValue {
            numerator: vec![Asset::SOL(pool_account.total_lamports)],
            denominator: pool_account.pool_token_supply,
        })
    }
}
