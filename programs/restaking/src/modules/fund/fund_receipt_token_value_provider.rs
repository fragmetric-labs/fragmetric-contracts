use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::errors::ErrorCode;
use crate::modules::fund::FundAccount;
use crate::modules::pricing::{Asset, TokenPricingSource, TokenValue, TokenValueProvider};

pub struct FundReceiptTokenValueProvider;

impl TokenValueProvider for FundReceiptTokenValueProvider {
    #[inline(never)]
    fn resolve_underlying_assets<'info>(
        self,
        token_mint: &Pubkey,
        pricing_source_accounts: &[&'info AccountInfo<'info>],
    ) -> Result<TokenValue> {
        #[cfg(debug_assertions)]
        require_eq!(pricing_source_accounts.len(), 1);

        let fund_account = Account::<FundAccount>::try_from(pricing_source_accounts[0])?;

        require_keys_eq!(fund_account.receipt_token_mint, *token_mint);

        let mut assets = Vec::new();

        // sol_operation_reserved_amount
        assets.push(Asset::SOL(fund_account.sol_operation_reserved_amount));

        // lst_operation_reserved_amount + lst_operating_amount (pending unstaking)
        for supported_token in &fund_account.supported_tokens {
            assets.push(Asset::TOKEN(
                supported_token.mint,
                Some(supported_token.pricing_source.clone()),
                supported_token.operation_reserved_amount + supported_token.operating_amount,
            ));
        }

        // nt_operation_reserved_amount
        if let Some(normalized_token) = &fund_account.normalized_token {
            assets.push(Asset::TOKEN(
                normalized_token.mint,
                Some(TokenPricingSource::FragmetricNormalizedTokenPool {
                    address: normalized_token.pool,
                }),
                normalized_token.operation_reserved_amount,
            ));
        }

        // TODO v0.3/operation: need to reflect vrt_operation_reserved/operating amount to pricing

        Ok(TokenValue {
            numerator: assets,
            denominator: fund_account.receipt_token_supply_amount,
        })
    }
}
