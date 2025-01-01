use anchor_lang::prelude::*;
use jito_bytemuck::AccountDeserialize;
use jito_vault_core::vault::Vault;

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::modules::pricing::{Asset, TokenValue, TokenValueProvider};
use crate::modules::restaking::JitoRestakingVaultService;

pub struct JitoRestakingVaultValueProvider;

impl TokenValueProvider for JitoRestakingVaultValueProvider {
    #[inline(never)]
    fn resolve_underlying_assets<'info>(
        self,
        token_value_to_update: &mut TokenValue,
        token_mint: &Pubkey,
        pricing_source_accounts: &[&'info AccountInfo<'info>],
    ) -> Result<()> {
        require_eq!(pricing_source_accounts.len(), 1);

        let vault = JitoRestakingVaultService::deserialize_vault(pricing_source_accounts[0])?;
        require_keys_eq!(vault.vrt_mint, *token_mint);

        token_value_to_update.numerator.clear();
        token_value_to_update.numerator.reserve_exact(1);

        token_value_to_update.numerator.extend([Asset::Token(
            vault.supported_mint,
            None,
            vault.tokens_deposited(),
        )]);
        token_value_to_update.denominator = vault.vrt_supply();

        Ok(())
    }
}
