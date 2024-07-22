use anchor_lang::prelude::*;

use crate::fund::*;

#[account]
#[derive(InitSpace)]
pub struct Fund {
    pub admin: Pubkey,                  // 32
    pub default_protocol_fee_rate: u16, // 2
    pub receipt_token_mint: Pubkey, // 32
    #[max_len(20)]
    pub tokens: Vec<TokenInfo>,
    // pub receipt_token_lock_account: Pubkey, // 32
    pub sol_amount_in: u128, // 16
}
