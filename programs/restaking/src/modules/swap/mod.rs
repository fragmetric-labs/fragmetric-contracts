mod orca_dex_liquidity_pool_service;
mod orca_dex_liquidity_pool_value_provider;
mod token_swap_source;

use anchor_lang::prelude::*;

pub use orca_dex_liquidity_pool_service::*;
pub use orca_dex_liquidity_pool_value_provider::*;
pub use token_swap_source::*;

/// Validate swap source account based on the owner(swap source program).
///
/// returns token swap source
pub fn validate_swap_source<'info>(
    swap_source_account: &'info AccountInfo<'info>,
    from_token_mint: &AccountInfo,
    to_token_mint: &AccountInfo,
) -> Result<TokenSwapSource> {
    match swap_source_account.owner {
        &whirlpool_cpi::whirlpool::ID_CONST => {
            OrcaDEXLiquidityPoolService::validate_pool_token(
                swap_source_account,
                from_token_mint,
                to_token_mint,
            )?;
            Ok(TokenSwapSource::OrcaDEXLiquidityPool {
                address: swap_source_account.key(),
            })
        }
        _ => err!(ErrorCode::AccountOwnedByWrongProgram)?,
    }
}
