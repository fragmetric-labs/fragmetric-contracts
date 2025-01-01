use anchor_lang::prelude::*;

use crate::modules::pricing::{Asset, TokenValue, TokenValueProvider};

use super::MarinadeStakePoolService;

pub struct MarinadeStakePoolValueProvider;

impl TokenValueProvider for MarinadeStakePoolValueProvider {
    #[inline(never)]
    fn resolve_underlying_assets<'info>(
        self,
        token_value_to_update: &mut TokenValue,
        token_mint: &Pubkey,
        pricing_source_accounts: &[&'info AccountInfo<'info>],
    ) -> Result<()> {
        require_eq!(pricing_source_accounts.len(), 1);

        // ref: https://docs.rs/marinade-cpi/latest/marinade_cpi/state/struct.State.html
        let state = MarinadeStakePoolService::deserialize_pool_account(pricing_source_accounts[0])?;
        require_keys_eq!(state.msol_mint, *token_mint);

        let total_cooling_down =
            state.stake_system.delayed_unstake_cooling_down + state.emergency_cooling_down;

        let total_lamports_under_control = state.validator_system.total_active_balance
            + total_cooling_down
            + state.available_reserve_balance;

        let total_value_staked_lamports =
            total_lamports_under_control.saturating_sub(state.circulating_ticket_balance);

        token_value_to_update.numerator.clear();
        token_value_to_update.numerator.reserve_exact(1);

        token_value_to_update
            .numerator
            .extend([Asset::SOL(total_value_staked_lamports)]);
        token_value_to_update.denominator = state.msol_supply;

        Ok(())
    }
}
