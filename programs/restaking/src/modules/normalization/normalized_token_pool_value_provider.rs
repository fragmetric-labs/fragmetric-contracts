use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::modules::pricing::{Asset, TokenValue, TokenValueProvider};

use super::*;

pub struct NormalizedTokenPoolValueProvider;

impl TokenValueProvider for NormalizedTokenPoolValueProvider {
    #[inline(never)]
    fn resolve_underlying_assets<'info>(
        self,
        pricing_source_accounts: &[&'info AccountInfo<'info>],
    ) -> Result<TokenValue> {
        require_eq!(pricing_source_accounts.len(), 2);

        let normalized_token_mint = InterfaceAccount::<Mint>::try_from(pricing_source_accounts[0])?;
        let normalized_token_pool_account =
            Account::<NormalizedTokenPoolAccount>::try_from(pricing_source_accounts[1])?;

        Ok(TokenValue {
            numerator: normalized_token_pool_account
                .supported_tokens
                .iter()
                .filter(|supported_token| supported_token.locked_amount > 0)
                .map(|supported_token| {
                    Asset::TOKEN(supported_token.mint, None, supported_token.locked_amount)
                })
                .collect(),
            denominator: normalized_token_mint.supply,
        })
    }
}
