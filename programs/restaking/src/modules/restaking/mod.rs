pub mod jito_restaking_vault_service;
pub mod jito_restaking_vault_value_provider;

pub use jito_restaking_vault_service::*;
pub use jito_restaking_vault_value_provider::*;

use anchor_lang::prelude::*;

use crate::constants::{JITO_VAULT_PROGRAM_ID, SOLV_PROGRAM_ID};
use crate::modules::pricing::TokenPricingSource;

/// Validate vault account based on the owner(vault program).
///
/// returns pricing source
pub(in crate::modules) fn validate_vault(
    vault_account: &AccountInfo,
    vault_supported_token_mint: &AccountInfo,
    vault_receipt_token_mint: &AccountInfo,
) -> Result<TokenPricingSource> {
    match vault_account.owner {
        &JITO_VAULT_PROGRAM_ID => {
            JitoRestakingVaultService::validate_vault(
                vault_account,
                vault_supported_token_mint,
                vault_receipt_token_mint,
            )?;
            Ok(TokenPricingSource::JitoRestakingVault {
                address: vault_account.key(),
            })
        }
        // TODO/v0.7.0: deal with solv vault if needed
        &SOLV_PROGRAM_ID => Ok(TokenPricingSource::SolvBTCVault {
            address: vault_account.key(),
        }),
        _ => err!(error::ErrorCode::AccountOwnedByWrongProgram)?,
    }
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
