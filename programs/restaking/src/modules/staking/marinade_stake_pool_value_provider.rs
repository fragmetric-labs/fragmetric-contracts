use crate::constants;
use crate::modules::pricing::{Asset, TokenPricingSource, TokenValue, TokenValueProvider};
use crate::utils;
use anchor_lang::prelude::*;
use marinade_cpi::state::State as MarinadeStakePoolAccount;

pub struct MarinadeStakePoolValueProvider;

impl TokenValueProvider for MarinadeStakePoolValueProvider {
    fn resolve_underlying_assets(
        _token_pricing_source: &TokenPricingSource,
        pricing_source_accounts: Vec<&AccountInfo>,
    ) -> Result<TokenValue> {
        require_eq!(pricing_source_accounts.len(), 1);

        // ref: https://docs.rs/marinade-cpi/latest/marinade_cpi/state/struct.State.html
        let pool_account = MarinadeStakePoolAccount::try_deserialize(
            &mut &**pricing_source_accounts[0].try_borrow_data()?,
        )
        .map_err(|_| error!(ErrorCode::AccountDidNotDeserialize))?;

        #[cfg(feature = "devnet")]
        require_keys_eq!(pool_account.msol_mint, constants::MAINNET_MSOL_MINT_ADDRESS);
        #[cfg(not(feature = "devnet"))]
        require_keys_eq!(pool_account.msol_mint, constants::DEVNET_MSOL_MINT_ADDRESS);

        let total_cooling_down =
            pool_account.stake_system.delayed_unstake_cooling_down + pool_account.emergency_cooling_down;

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
