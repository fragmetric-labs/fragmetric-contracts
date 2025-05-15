use anchor_lang::prelude::*;

use crate::modules::pricing::{Asset, TokenValue, TokenValueProvider};

use super::*;

pub struct NormalizedTokenPoolValueProvider;

impl TokenValueProvider for NormalizedTokenPoolValueProvider {
    #[inline(never)]
    fn resolve_underlying_assets<'info>(
        self,
        token_mint: &Pubkey,
        pricing_source_accounts: &[&'info AccountInfo<'info>],
        result: &mut TokenValue,
    ) -> Result<()> {
        require_eq!(pricing_source_accounts.len(), 1);

        let normalized_token_pool_account =
            NormalizedTokenPoolService::deserialize_pool_account(pricing_source_accounts[0])?;
        require_keys_eq!(
            normalized_token_pool_account.normalized_token_mint,
            *token_mint
        );

        result.numerator.clear();
        result
            .numerator
            .reserve_exact(NormalizedTokenPoolAccount::MAX_SUPPORTED_TOKENS_SIZE);

        result
            .numerator
            .extend(
                normalized_token_pool_account
                    .supported_tokens
                    .iter()
                    .map(|supported_token| {
                        Asset::Token(
                            supported_token.mint,
                            Some(supported_token.pricing_source.clone()),
                            supported_token.locked_amount,
                        )
                    }),
            );
        result.denominator = normalized_token_pool_account.normalized_token_supply_amount;

        Ok(())
    }
}
