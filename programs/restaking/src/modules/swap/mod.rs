mod orca_dex_liquidity_pool_service;
mod orca_dex_liquidity_pool_value_provider;
mod token_swap_source;

pub use orca_dex_liquidity_pool_service::*;
pub use orca_dex_liquidity_pool_value_provider::*;
pub use token_swap_source::*;

use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::errors::ErrorCode;
use crate::modules::pricing::TokenPricingSource;

/// Validate liquidity pool pricing source
pub(in crate::modules) fn validate_pricing_source<'info>(
    pricing_source: &TokenPricingSource,
    pool_account: &'info AccountInfo<'info>,
    from_token_mint: &Pubkey,
    to_token_mint: &Pubkey,
) -> Result<()> {
    match pricing_source {
        TokenPricingSource::OrcaDEXLiquidityPool { address } => {
            require_keys_eq!(*address, pool_account.key());
            OrcaDEXLiquidityPoolService::validate_liquidity_pool(
                pool_account,
                from_token_mint,
                to_token_mint,
            )?
        }
        TokenPricingSource::SPLStakePool { .. }
        | TokenPricingSource::MarinadeStakePool { .. }
        | TokenPricingSource::JitoRestakingVault { .. }
        | TokenPricingSource::FragmetricNormalizedTokenPool { .. }
        | TokenPricingSource::FragmetricRestakingFund { .. }
        | TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. }
        | TokenPricingSource::PeggedToken { .. }
        | TokenPricingSource::SolvBTCVault { .. }
        | TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. }
        | TokenPricingSource::VirtualVault { .. }
        | TokenPricingSource::DriftVault { .. } => err!(ErrorCode::UnexpectedPricingSourceError)?,
        #[cfg(all(test, not(feature = "idl-build")))]
        TokenPricingSource::Mock { .. } => err!(ErrorCode::UnexpectedPricingSourceError)?,
    }

    Ok(())
}

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
            OrcaDEXLiquidityPoolService::validate_liquidity_pool(
                swap_source_account,
                &from_token_mint.key(),
                &to_token_mint.key(),
            )?
        }
    }

    Ok(())
}

trait ValidateLiquidityPool {
    fn validate_liquidity_pool<'info>(
        pool_account: &'info AccountInfo<'info>,
        from_token_mint: &Pubkey,
        to_token_mint: &Pubkey,
    ) -> Result<()>;
}
