use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::errors::ErrorCode;

use crate::modules::fund::*;

/// Price = SOL value for denominated amount of 1 token
///
/// Receipt token price might be outdated.
pub(in crate::modules) fn receipt_token_price(
    receipt_token_mint: &InterfaceAccount<Mint>,
    fund_account: &Account<FundAccount>,
) -> Result<u64> {
    crate::utils::proportional_amount(
        10u64
            .checked_pow(receipt_token_mint.decimals as u32)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?,
        fund_account.total_sol_value()?,
        receipt_token_mint.supply,
    )
    .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
}
