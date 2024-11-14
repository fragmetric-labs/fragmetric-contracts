use crate::modules::fund::FundAccount;
use crate::modules::normalization::NormalizedTokenPoolAccount;
use crate::modules::pricing::{Asset, TokenPricingSource, TokenValue, TokenValueProvider};
use crate::utils;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

pub struct NormalizedTokenPoolValueProvider<'a> {
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> TokenValueProvider for NormalizedTokenPoolValueProvider<'a> {
    fn resolve_underlying_assets(
        _token_pricing_source: &TokenPricingSource,
        pricing_source_accounts: Vec<&AccountInfo>,
    ) -> Result<TokenValue> {
        require_eq!(pricing_source_accounts.len(), 2);

        // CHECKED: to narrow down 'info lifetime of AccountInfos to 'a due to signature of try_from. below AccountInfos are dropped after this function returns.
        let normalized_token_mint_info = pricing_source_accounts[0].clone();
        let normalized_token_mint = InterfaceAccount::<Mint>::try_from(unsafe {
            std::mem::transmute::<_, &'a AccountInfo<'a>>(&normalized_token_mint_info)
        })?;
        let normalized_token_pool_account_info = pricing_source_accounts[1].clone();
        let normalized_token_pool_account =
            Account::<NormalizedTokenPoolAccount>::try_from(unsafe {
                std::mem::transmute::<_, &'a AccountInfo<'a>>(&normalized_token_pool_account_info)
            })?;

        Ok(TokenValue {
            numerator: normalized_token_pool_account
                .supported_tokens
                .iter()
                .filter(|supported_token| supported_token.get_locked_amount() > 0)
                .map(|supported_token| {
                    Asset::TOKEN(
                        supported_token.get_mint(),
                        None,
                        supported_token.get_locked_amount(),
                    )
                })
                .collect(),
            denominator: normalized_token_mint.supply,
        })
    }
}
