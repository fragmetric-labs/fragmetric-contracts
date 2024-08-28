use anchor_lang::prelude::*;

use crate::{Fund, SupportedTokenInfo};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FundInfo {
    pub receipt_token_mint: Pubkey,
    pub receipt_token_price: u64,
    pub receipt_token_supply_amount: u64,
    pub supported_tokens: Vec<SupportedTokenInfo>,
    pub sol_operation_reserved_amount: u64,
    pub sol_withdrawal_reserved_amount: u64,
    pub sol_withdrawal_fee_rate: f32,
    pub sol_withdrawal_enabled: bool,
}

impl FundInfo {
    pub fn new_from_fund(
        fund: &Fund,
        receipt_token_price: u64,
        receipt_token_supply_amount: u64,
    ) -> Self {
        FundInfo {
            receipt_token_mint: fund.receipt_token_mint,
            receipt_token_price,
            receipt_token_supply_amount,
            supported_tokens: fund.supported_tokens.clone(),
            sol_operation_reserved_amount: fund.sol_operation_reserved_amount,
            sol_withdrawal_reserved_amount: fund.withdrawal_status.reserved_fund.sol_remaining,
            sol_withdrawal_fee_rate: fund.withdrawal_status.sol_withdrawal_fee_rate_f32(),
            sol_withdrawal_enabled: fund.withdrawal_status.withdrawal_enabled_flag,
        }
    }
}
