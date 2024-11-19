use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::modules::pricing::{Asset, TokenPricingSource, TokenValue, TokenValueProvider};

use super::*;

pub struct NormalizedTokenPoolValueProvider;

impl TokenValueProvider for NormalizedTokenPoolValueProvider {
    #[inline(never)]
    fn resolve_underlying_assets<'a, 'info: 'a>(
        _token_pricing_source: &TokenPricingSource,
        pricing_source_accounts: Vec<&'a AccountInfo<'info>>,
    ) -> Result<TokenValue> {
        require_eq!(pricing_source_accounts.len(), 2);

        // CHECKED: to narrow down 'info lifetime of `AccountInfo`s to 'a due to signature of try_from.
        // below `AccountInfo`s are dropped when this function returns.
        let normalized_token_mint_info = pricing_source_accounts[0];
        let normalized_token_mint = InterfaceAccount::<'a, Mint>::try_from(unsafe {
            std::mem::transmute(&normalized_token_mint_info)
        })?;
        let normalized_token_pool_account_info = pricing_source_accounts[1].clone();
        let normalized_token_pool_account =
            Account::<'a, NormalizedTokenPoolAccount>::try_from(unsafe {
                std::mem::transmute(&normalized_token_pool_account_info)
            })?;

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
