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
        token_value_to_update: &mut TokenValue,
        token_mint: &Pubkey,
        pricing_source_accounts: &[&'info AccountInfo<'info>],
    ) -> Result<()> {
        require_eq!(pricing_source_accounts.len(), 1);

        let stake_pool =
            <SPLStakePoolService>::deserialize_pool_account(pricing_source_accounts[0])?;
        require_keys_eq!(stake_pool.pool_mint, *token_mint);

        token_value_to_update.numerator.clear();
        token_value_to_update.numerator.reserve_exact(1);

        token_value_to_update
            .numerator
            .extend([Asset::SOL(stake_pool.total_lamports)]);
        token_value_to_update.denominator = stake_pool.pool_token_supply;

        Ok(())
    }
}
