use anchor_lang::prelude::*;

use crate::TokenInfo;

#[event]
pub struct FundInfo {
    pub admin: Pubkey,
    pub lrt_mint: Pubkey,
    pub supported_tokens: Vec<TokenInfo>,
    pub sol_amount_in: u128,
    pub sol_reserved_amount: u128,
    pub sol_withdrawal_fee_rate: f32,
    pub sol_withdrawal_enabled: bool,
}
