mod marinade_stake_pool_service;
mod marinade_stake_pool_value_provider;
mod sanctum_multi_validator_spl_stake_pool_service;
mod sanctum_single_validator_spl_stake_pool_service;
mod spl_stake_pool_service;
mod spl_stake_pool_value_provider;

pub use marinade_stake_pool_service::*;
pub use marinade_stake_pool_value_provider::*;
pub use sanctum_multi_validator_spl_stake_pool_service::*;
pub use sanctum_single_validator_spl_stake_pool_service::*;
pub use spl_stake_pool_service::*;
pub use spl_stake_pool_value_provider::*;

use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::errors::ErrorCode;
use crate::modules::pricing::TokenPricingSource;

/// Validate stake pool account
pub fn validate_stake_pool<'info>(
    pricing_source: &TokenPricingSource,
    pool_account: &'info AccountInfo<'info>,
    pool_token_mint: &InterfaceAccount<'info, Mint>,
) -> Result<()> {
    #[deny(clippy::wildcard_enum_match_arm)]
    match pricing_source {
        TokenPricingSource::SPLStakePool { address } => {
            require_keys_eq!(*address, pool_account.key());
            <SPLStakePoolService>::validate_stake_pool(pool_account, pool_token_mint)?
        }
        TokenPricingSource::MarinadeStakePool { address } => {
            require_keys_eq!(*address, pool_account.key());
            MarinadeStakePoolService::validate_stake_pool(pool_account, pool_token_mint)?
        }
        TokenPricingSource::SanctumSingleValidatorSPLStakePool { address } => {
            require_keys_eq!(*address, pool_account.key());
            SanctumSingleValidatorSPLStakePoolService::validate_stake_pool(
                pool_account,
                pool_token_mint,
            )?
        }
        TokenPricingSource::SanctumMultiValidatorSPLStakePool { address } => {
            require_keys_eq!(*address, pool_account.key());
            SanctumMultiValidatorSPLStakePoolService::validate_stake_pool(
                pool_account,
                pool_token_mint,
            )?
        }
        // otherwise fails
        TokenPricingSource::JitoRestakingVault { .. }
        | TokenPricingSource::FragmetricNormalizedTokenPool { .. }
        | TokenPricingSource::FragmetricRestakingFund { .. }
        | TokenPricingSource::OrcaDEXLiquidityPool { .. }
        | TokenPricingSource::PeggedToken { .. }
        | TokenPricingSource::SolvBTCVault { .. }
        | TokenPricingSource::VirtualVault { .. } => err!(ErrorCode::UnexpectedPricingSourceError)?,
        #[cfg(all(test, not(feature = "idl-build")))]
        TokenPricingSource::Mock { .. } => err!(ErrorCode::UnexpectedPricingSourceError)?,
    }

    Ok(())
}

trait ValidateStakePool {
    fn validate_stake_pool<'info>(
        pool_account: &'info AccountInfo<'info>,
        pool_token_mint: &InterfaceAccount<'info, Mint>,
    ) -> Result<()>;
}
