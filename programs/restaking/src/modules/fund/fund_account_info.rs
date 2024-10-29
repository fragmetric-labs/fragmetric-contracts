use anchor_lang::prelude::*;

use super::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FundAccountInfo {
    receipt_token_mint: Pubkey,
    receipt_token_price: u64,
    receipt_token_supply_amount: u64,
    supported_tokens: Vec<SupportedTokenInfo>,
    sol_capacity_amount: u64,
    sol_accumulated_deposit_amount: u64,
    sol_operation_reserved_amount: u64,
    sol_withdrawal_reserved_amount: u64,
    sol_withdrawal_fee_rate: f32,
    withdrawal_enabled: bool,
    withdrawal_last_completed_batch_id: u64,
}

impl FundAccountInfo {
    // TODO visibility is currently set to `in crate::modules` due to operator module - change to `super`
    pub(in crate::modules) fn from(
        fund_account: &FundAccount,
        receipt_token_price: u64,
        receipt_token_supply_amount: u64,
    ) -> Self {
        FundAccountInfo {
            receipt_token_mint: fund_account.receipt_token_mint,
            receipt_token_price,
            receipt_token_supply_amount,
            supported_tokens: fund_account.get_supported_tokens_iter().cloned().collect(),
            sol_capacity_amount: fund_account.get_sol_capacity_amount(),
            sol_accumulated_deposit_amount: fund_account.get_sol_accumulated_deposit_amount(),
            sol_operation_reserved_amount: fund_account.get_sol_operation_reserved_amount(),
            sol_withdrawal_reserved_amount: fund_account
                .withdrawal
                .get_sol_withdrawal_reserved_amount(),
            sol_withdrawal_fee_rate: fund_account.withdrawal.get_sol_withdrawal_fee_rate_as_f32(),
            withdrawal_enabled: fund_account.withdrawal.get_withdrawal_enabled_flag(),
            withdrawal_last_completed_batch_id: fund_account
                .withdrawal
                .get_last_completed_batch_id(),
        }
    }
}
