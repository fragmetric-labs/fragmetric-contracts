use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::modules::pricing::{Asset, TokenValue, TokenValueProvider};

use super::*;

pub struct NormalizedTokenPoolValueProvider;

impl TokenValueProvider for NormalizedTokenPoolValueProvider {
    #[inline(never)]
    fn resolve_underlying_assets<'info>(
        self,
        token_value_to_update: &mut TokenValue,
        token_mint: &Pubkey,
        pricing_source_accounts: &[&'info AccountInfo<'info>],
    ) -> Result<()> {
        require_eq!(pricing_source_accounts.len(), 1);

        let normalized_token_pool_account =
            NormalizedTokenPoolService::deserialize_pool_account(pricing_source_accounts[0])?;
        require_keys_eq!(
            normalized_token_pool_account.normalized_token_mint,
            *token_mint
        );

        token_value_to_update.numerator.clear();
        token_value_to_update
            .numerator
            .reserve_exact(normalized_token_pool_account.supported_tokens.len());

        token_value_to_update.numerator.extend(
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
        token_value_to_update.denominator =
            normalized_token_pool_account.normalized_token_supply_amount;

        Ok(())
    }
}
