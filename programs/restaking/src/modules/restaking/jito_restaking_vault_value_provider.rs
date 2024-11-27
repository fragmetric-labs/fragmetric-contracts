use anchor_lang::prelude::*;
use jito_bytemuck::AccountDeserialize;
use jito_vault_core::vault::Vault;

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::modules::pricing::{Asset, TokenValue, TokenValueProvider};

pub struct JitoRestakingVaultValueProvider;

impl TokenValueProvider for JitoRestakingVaultValueProvider {
    #[inline(never)]
    fn resolve_underlying_assets<'info>(
        self,
        token_mint: &Pubkey,
        pricing_source_accounts: &[&'info AccountInfo<'info>],
    ) -> Result<TokenValue> {
        #[cfg(debug_assertions)]
        require_eq!(pricing_source_accounts.len(), 1);

        let vault_data = pricing_source_accounts[0].data.borrow();
        let vault_account = Vault::try_from_slice_unchecked(vault_data.as_ref())?;

        require_keys_eq!(vault_account.vrt_mint, *token_mint);

        Ok(TokenValue {
            numerator: vec![Asset::TOKEN(
                vault_account.supported_mint,
                None,
                vault_account.tokens_deposited(),
            )],
            denominator: vault_account.vrt_supply(),
        })
    }
}
