mod orca_dex_liquidity_pool_service;
mod orca_dex_liquidity_pool_value_provider;
mod token_swap_source;

pub use orca_dex_liquidity_pool_service::*;
pub use orca_dex_liquidity_pool_value_provider::*;
pub use token_swap_source::*;

use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

/// Validate swap source account
pub(in crate::modules) fn validate_swap_source<'info>(
    swap_source: &TokenSwapSource,
    swap_source_account: &'info AccountInfo<'info>,
    from_token_mint: &InterfaceAccount<Mint>,
    to_token_mint: &InterfaceAccount<Mint>,
) -> Result<()> {
    match swap_source {
        TokenSwapSource::OrcaDEXLiquidityPool { address } => {
            require_keys_eq!(*address, swap_source_account.key());
            OrcaDEXLiquidityPoolService::validate_swap_source(
                swap_source_account,
                from_token_mint,
                to_token_mint,
            )?
        }
    }

    Ok(())
}

trait ValidateSwapSource {
    fn validate_swap_source<'info>(
        swap_source_account: &'info AccountInfo<'info>,
        from_token_mint: &InterfaceAccount<Mint>,
        to_token_mint: &InterfaceAccount<Mint>,
    ) -> Result<()>;
}
