use anchor_lang::prelude::*;
use marinade_cpi::state::State;

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::modules::pricing::{Asset, TokenValue, TokenValueProvider};

pub struct MarinadeStakePoolValueProvider;

impl TokenValueProvider for MarinadeStakePoolValueProvider {
    #[inline(never)]
    fn resolve_underlying_assets<'info>(
        self,
        token_mint: &Pubkey,
        pricing_source_accounts: &[&'info AccountInfo<'info>],
    ) -> Result<TokenValue> {
        #[cfg(debug_assertions)]
        require_eq!(pricing_source_accounts.len(), 1);

        // ref: https://docs.rs/marinade-cpi/latest/marinade_cpi/state/struct.State.html
        let pool_account = Account::<State>::try_from(pricing_source_accounts[0])?;

        require_keys_eq!(pool_account.msol_mint, *token_mint);

        let total_cooling_down = pool_account.stake_system.delayed_unstake_cooling_down
            + pool_account.emergency_cooling_down;

        let total_lamports_under_control = pool_account.validator_system.total_active_balance
            + total_cooling_down
            + pool_account.available_reserve_balance;

        let total_value_staked_lamports =
            total_lamports_under_control.saturating_sub(pool_account.circulating_ticket_balance);

        Ok(TokenValue {
            numerator: vec![Asset::SOL(total_value_staked_lamports)],
            denominator: pool_account.msol_supply,
        })
    }
}
