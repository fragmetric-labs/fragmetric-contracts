use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::modules::pricing::{Asset, TokenValue, TokenValueProvider};

use super::*;

pub struct NormalizedTokenPoolValueProvider;

impl TokenValueProvider for NormalizedTokenPoolValueProvider {
    #[inline(never)]
    fn resolve_underlying_assets<'info>(
        self,
        token_mint: &Pubkey,
        pricing_source_accounts: &[&'info AccountInfo<'info>],
    ) -> Result<TokenValue> {
        require_eq!(pricing_source_accounts.len(), 1);

        let normalized_token_pool_account =
            Account::<NormalizedTokenPoolAccount>::try_from(pricing_source_accounts[0])?;

        require_keys_eq!(
            normalized_token_pool_account.normalized_token_mint,
            *token_mint
        );

        Ok(TokenValue {
            numerator: normalized_token_pool_account
                .supported_tokens
                .iter()
                .filter(|supported_token| supported_token.locked_amount > 0)
                .map(|supported_token| {
                    Asset::Token(
                        supported_token.mint,
                        Some(supported_token.pricing_source.clone()),
                        supported_token.locked_amount,
                    )
                })
                .collect(),
            denominator: normalized_token_pool_account.normalized_token_supply_amount,
        })
    }
}
