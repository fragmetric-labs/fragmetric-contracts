use anchor_lang::prelude::*;

use crate::modules::pricing::{Asset, TokenValue, TokenValueProvider};

use super::MarinadeStakePoolService;

pub struct MarinadeStakePoolValueProvider;

impl TokenValueProvider for MarinadeStakePoolValueProvider {
    #[inline(never)]
    fn resolve_underlying_assets<'info>(
        self,
        token_mint: &Pubkey,
        pricing_source_accounts: &[&'info AccountInfo<'info>],
        result: &mut TokenValue,
    ) -> Result<()> {
        require_eq!(pricing_source_accounts.len(), 1);

        // ref: https://docs.rs/marinade-cpi/latest/marinade_cpi/state/struct.State.html
        let state = MarinadeStakePoolService::deserialize_pool_account(pricing_source_accounts[0])?;
        require_keys_eq!(state.msol_mint, *token_mint);

        let total_virtual_staked_lamports =
            MarinadeStakePoolService::get_total_virtual_staked_lamports(&state);

        result.numerator.clear();
        result.numerator.reserve_exact(1);

        result
            .numerator
            .extend([Asset::SOL(total_virtual_staked_lamports)]);
        result.denominator = state.msol_supply;

        Ok(())
    }
}
