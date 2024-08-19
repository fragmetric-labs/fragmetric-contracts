use anchor_lang::prelude::*;

use crate::{Fund, TokenInfo};

#[event]
pub struct FundInfo {
    pub lrt_mint: Pubkey,
    pub supported_tokens: Vec<TokenInfo>,
    pub sol_amount_in: u64,
    pub sol_reserved_amount: u64,
    pub sol_withdrawal_fee_rate: f32,
    pub sol_withdrawal_enabled: bool,
}

impl FundInfo {
    pub fn new_from_fund(fund: &Fund) -> Self {
        FundInfo {
            lrt_mint: fund.receipt_token_mint,
            supported_tokens: fund.supported_tokens.clone(),
            sol_amount_in: fund.sol_amount_in,
            sol_reserved_amount: fund.withdrawal_status.reserved_fund.sol_remaining,
            sol_withdrawal_fee_rate: fund.withdrawal_status.sol_withdrawal_fee_rate_f32(),
            sol_withdrawal_enabled: fund.withdrawal_status.withdrawal_enabled_flag,
        }
    }
}
