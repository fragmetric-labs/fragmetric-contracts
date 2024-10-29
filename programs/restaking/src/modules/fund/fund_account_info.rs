use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use crate::modules::fund;
use super::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FundAccountInfo {
    receipt_token_mint: Pubkey,
    one_receipt_token_as_sol: u64,
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
        fund_account: &Account<FundAccount>,
        receipt_token_mint: &InterfaceAccount<Mint>,
    ) -> Self {
        FundAccountInfo {
            receipt_token_mint: fund_account.receipt_token_mint,
            one_receipt_token_as_sol: get_one_receipt_token_as_sol(receipt_token_mint, fund_account).unwrap(),
            receipt_token_supply_amount: receipt_token_mint.supply,
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
