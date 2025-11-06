pub mod jito_restaking_vault_service;
pub mod jito_restaking_vault_value_provider;
pub mod solv_btc_vault_service;
pub mod solv_btc_vault_value_provider;
pub mod virtual_vault_service;

pub use jito_restaking_vault_service::*;
pub use jito_restaking_vault_value_provider::*;
pub use solv_btc_vault_service::*;
pub use solv_btc_vault_value_provider::*;
pub use virtual_vault_service::*;

use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::errors::ErrorCode;
use crate::modules::pricing::TokenPricingSource;

/// Validate restaking vault pricing source
pub(in crate::modules) fn validate_pricing_source<'info>(
    pricing_source: &TokenPricingSource,
    vault_account: &'info AccountInfo<'info>,
    vault_supported_token_mint: &InterfaceAccount<Mint>,
    vault_receipt_token_mint: &InterfaceAccount<Mint>,
    fund_account: &AccountInfo,
) -> Result<()> {
    match pricing_source {
        TokenPricingSource::JitoRestakingVault { address } => {
            require_keys_eq!(*address, vault_account.key());
            JitoRestakingVaultService::validate_vault(
                vault_account,
                vault_supported_token_mint,
                vault_receipt_token_mint,
                fund_account,
            )?
        }
        TokenPricingSource::SolvBTCVault { address } => {
            require_keys_eq!(*address, vault_account.key());
            SolvBTCVaultService::validate_vault(
                vault_account,
                vault_supported_token_mint,
                vault_receipt_token_mint,
                fund_account,
            )?
        }
        TokenPricingSource::VirtualVault { address } => {
            require_keys_eq!(*address, vault_account.key());
            VirtualVaultService::validate_vault(
                vault_account,
                vault_supported_token_mint,
                vault_receipt_token_mint,
                fund_account,
            )?
        }
        TokenPricingSource::DriftVault { .. } => {
            todo!()
        }
        // otherwise fails
        TokenPricingSource::SPLStakePool { .. }
        | TokenPricingSource::MarinadeStakePool { .. }
        | TokenPricingSource::FragmetricNormalizedTokenPool { .. }
        | TokenPricingSource::FragmetricRestakingFund { .. }
        | TokenPricingSource::OrcaDEXLiquidityPool { .. }
        | TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. }
        | TokenPricingSource::PeggedToken { .. }
        | TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. } => {
            err!(ErrorCode::UnexpectedPricingSourceError)?
        }
        #[cfg(all(test, not(feature = "idl-build")))]
        TokenPricingSource::Mock { .. } => err!(ErrorCode::UnexpectedPricingSourceError)?,
    }

    Ok(())
}

pub(in crate::modules) trait ValidateVault {
    fn validate_vault<'info>(
        vault_account: &'info AccountInfo<'info>,
        vault_supported_token_mint: &InterfaceAccount<Mint>,
        vault_receipt_token_mint: &InterfaceAccount<Mint>,
        fund_account: &AccountInfo,
    ) -> Result<()>;
}

/// Validate delegation based on the owner(vault program).
///
/// returns [delegation_index, delegated_amount, undelegating_amount],
/// while delegation index is optional.
pub(in crate::modules) fn validate_vault_operator_delegation(
    vault_operator_delegation: &AccountInfo,
    vault_account: &AccountInfo,
    operator: &AccountInfo,
) -> Result<(Option<u8>, u64, u64)> {
    match vault_account.owner {
        &JITO_VAULT_PROGRAM_ID => {
            let (
                delegation_index,
                delegated_amount,
                undelegation_requested_amount,
                undelegating_amount,
            ) = JitoRestakingVaultService::validate_vault_operator_delegation(
                vault_operator_delegation,
                vault_account,
                operator,
            )?;

            require_gte!(u8::MAX as u64, delegation_index);

            Ok((
                Some(delegation_index as u8),
                delegated_amount,
                undelegation_requested_amount + undelegating_amount,
            ))
        }
        _ => err!(error::ErrorCode::AccountOwnedByWrongProgram),
    }
}
