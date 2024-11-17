use crate::errors::ErrorCode;
use crate::modules::fund::FundAccount;
use crate::modules::normalization::NormalizedTokenPoolAccount;
use crate::modules::pricing::{Asset, TokenPricingSource, TokenValue, TokenValueProvider};
use crate::utils;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

pub struct FundReceiptTokenValueProvider<'a> {
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> TokenValueProvider for FundReceiptTokenValueProvider<'a> {
    fn resolve_underlying_assets(
        _token_pricing_source: &TokenPricingSource,
        pricing_source_accounts: Vec<&AccountInfo>,
    ) -> Result<TokenValue> {
        require_eq!(pricing_source_accounts.len(), 2);

        // CHECKED: to narrow down 'info lifetime of AccountInfos to 'a due to signature of try_from. below AccountInfos are dropped after this function returns.
        let receipt_token_mint_info = pricing_source_accounts[0].clone();
        let receipt_token_mint = InterfaceAccount::<Mint>::try_from(unsafe {
            std::mem::transmute::<_, &'a AccountInfo<'a>>(&receipt_token_mint_info)
        })?;
        let fund_account_info = pricing_source_accounts[1].clone();
        let fund_account = Account::<FundAccount>::try_from(unsafe {
            std::mem::transmute::<_, &'a AccountInfo<'a>>(&fund_account_info)
        })?;

        let mut assets = Vec::new();
        for supported_token in &fund_account.supported_tokens {
            assets.push(Asset::TOKEN(
                supported_token.get_mint(),
                Some(supported_token.get_pricing_source()),
                supported_token
                    .get_operation_reserved_amount()
                    .checked_add(supported_token.get_operating_amount())
                    .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?,
            ));
        }
        assets.push(Asset::SOL(fund_account.sol_operation_reserved_amount));

        // TODO v0.3/operation: need to add the nt_operation_reserved + vrt_operation_reserved + pending unstaking lst + pending unrestaking vrt to pricing

        Ok(TokenValue {
            numerator: assets,
            denominator: receipt_token_mint.supply,
        })
    }
}
