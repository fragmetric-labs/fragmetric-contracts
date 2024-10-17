use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::errors::ErrorCode;

use crate::modules::fund::*;

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
