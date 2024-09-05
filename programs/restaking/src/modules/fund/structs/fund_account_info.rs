use anchor_lang::prelude::*;

use crate::modules::fund::{FundAccount, SupportedTokenInfo};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FundAccountInfo {
    pub receipt_token_mint: Pubkey,
    pub receipt_token_price: u64,
    pub receipt_token_supply_amount: u64,
    pub supported_tokens: Vec<SupportedTokenInfo>,
    pub sol_accumulated_deposit_amount: u64,
    pub sol_operation_reserved_amount: u64,
    pub sol_withdrawal_reserved_amount: u64,
    pub sol_withdrawal_fee_rate: f32,
    pub withdrawal_enabled: bool,
    pub withdrawal_last_completed_batch_id: u64,
}

impl FundAccountInfo {
    pub fn new(
        fund: &FundAccount,
        receipt_token_price: u64,
        receipt_token_supply_amount: u64,
    ) -> Self {
        FundAccountInfo {
            receipt_token_mint: fund.receipt_token_mint,
            receipt_token_price,
            receipt_token_supply_amount,
            supported_tokens: fund.supported_tokens.clone(),
            sol_accumulated_deposit_amount: fund.sol_accumulated_deposit_amount,
            sol_operation_reserved_amount: fund.sol_operation_reserved_amount,
            sol_withdrawal_reserved_amount: fund.withdrawal_status.reserved_fund.sol_remaining,
            sol_withdrawal_fee_rate: fund.withdrawal_status.sol_withdrawal_fee_rate_f32(),
            withdrawal_enabled: fund.withdrawal_status.withdrawal_enabled_flag,
            withdrawal_last_completed_batch_id: fund.withdrawal_status.last_completed_batch_id,
        }
    }
}
