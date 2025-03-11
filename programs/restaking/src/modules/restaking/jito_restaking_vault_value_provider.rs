use anchor_lang::prelude::*;
use jito_bytemuck::AccountDeserialize;
use jito_vault_core::vault::Vault;

use crate::modules::pricing::{Asset, TokenValue, TokenValueProvider};

pub struct JitoRestakingVaultValueProvider;

impl TokenValueProvider for JitoRestakingVaultValueProvider {
    #[inline(never)]
    fn resolve_underlying_assets<'info>(
        self,
        token_mint: &Pubkey,
        pricing_source_accounts: &[&'info AccountInfo<'info>],
        result: &mut TokenValue,
    ) -> Result<()> {
        require_eq!(pricing_source_accounts.len(), 1);

        let vault = JitoRestakingVaultService::deserialize_vault(pricing_source_accounts[0])?;
        require_keys_eq!(vault.vrt_mint, *token_mint);

        result.numerator.clear();
        result.numerator.reserve_exact(1);

        result.numerator.extend([Asset::Token(
            vault.supported_mint,
            None,
            vault.tokens_deposited(),
        )]);
        result.denominator = vault.vrt_supply();

        Ok(())
    }
}
