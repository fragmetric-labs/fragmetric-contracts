use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::modules::pricing::TokenValue;

use super::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FundAccountInfo {
    receipt_token_mint: Pubkey,
    receipt_token_decimals: u8,
    receipt_token_supply_amount: u64,
    receipt_token_value: TokenValue,
    one_receipt_token_as_sol: u64,
    supported_tokens: Vec<SupportedToken>,
    sol_capacity_amount: u64,
    sol_accumulated_deposit_amount: u64,
    sol_operation_reserved_amount: u64,
    sol_withdrawal_reserved_amount: u64,
    sol_withdrawal_fee_rate: f32,
    withdrawal_enabled: bool,
    withdrawal_last_completed_batch_id: u64,
    next_operation_sequence: u16,
}

impl FundAccountInfo {
    // TODO v0.3/operation: visibility
    pub(in crate::modules) fn from(
        fund_account: &Account<FundAccount>,
    ) -> Self {
        FundAccountInfo {
            receipt_token_mint: fund_account.receipt_token_mint,
            receipt_token_decimals: fund_account.receipt_token_decimals,
            receipt_token_supply_amount: fund_account.receipt_token_supply_amount,
            receipt_token_value: fund_account.receipt_token_value.clone(),
            one_receipt_token_as_sol: fund_account.one_receipt_token_as_sol,
            supported_tokens: fund_account.supported_tokens.clone(),
            sol_capacity_amount: fund_account.sol_capacity_amount,
            sol_accumulated_deposit_amount: fund_account.sol_accumulated_deposit_amount,
            sol_operation_reserved_amount: fund_account.sol_operation_reserved_amount,
            sol_withdrawal_reserved_amount: fund_account.withdrawal.sol_withdrawal_reserved_amount,
            sol_withdrawal_fee_rate: fund_account.withdrawal.get_sol_withdrawal_fee_rate_as_f32(),
            withdrawal_enabled: fund_account.withdrawal.withdrawal_enabled_flag,
            withdrawal_last_completed_batch_id: fund_account.withdrawal.last_completed_batch_id,
            next_operation_sequence: fund_account.operation.next_sequence,
        }
    }
}
