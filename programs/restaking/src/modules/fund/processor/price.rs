use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::errors::ErrorCode;

use crate::modules::fund::*;
use crate::modules::pricing::{self, TokenPricingSourceMap};

/// Receipt token price might be outdated.
pub(in crate::modules) fn get_one_receipt_token_as_sol(
    receipt_token_mint: &InterfaceAccount<Mint>,
    fund_account: &Account<FundAccount>,
) -> Result<u64> {
    crate::utils::get_proportional_amount(
        10u64
            .checked_pow(receipt_token_mint.decimals as u32)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?,
        fund_account.get_assets_total_amount_as_sol()?,
        receipt_token_mint.supply,
    )
    .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
}

pub(in crate::modules) fn create_pricing_source_map<'info>(
    fund_account: &Account<FundAccount>,
    pricing_source_accounts: &'info [AccountInfo<'info>],
) -> Result<TokenPricingSourceMap<'info>> {
    let mut pricing_source_map = TokenPricingSourceMap::default();
    // just convert array to map (by its pubkey)
    let pricing_source_accounts = pricing_source_accounts
        .iter()
        .map(|account| (account.key(), account))
        .collect();

    for token in fund_account.get_supported_tokens_iter() {
        let mint = token.get_mint();
        let pricing_source = token.get_pricing_source();
        let accounts = pricing::find_related_pricing_source_accounts(
            &pricing_source,
            &pricing_source_accounts,
        )?;
        pricing_source_map.insert(mint, (pricing_source, accounts));
    }

    Ok(pricing_source_map)
}
