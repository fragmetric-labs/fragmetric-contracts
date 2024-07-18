use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Fund {
    pub admin: Pubkey,                  // 32
    pub default_protocol_fee_rate: u16, // 2
    #[max_len(20)] // 20개 허용
    pub whitelisted_tokens: Vec<Pubkey>, // approved lst address list, 32 * 20
    // pub total_deposited_amount: u128, // 토큰별 total deposited amount, 16
    #[max_len(20)] // 20개 허용
    pub lst_caps: Vec<u64>, // each lst's cap, 8 * 20
    pub receipt_token_mint: Pubkey, // 32
    // pub receipt_token_lock_account: Pubkey, // 32
    pub sol_amount_in: u128, // 16
    #[max_len(20)]
    pub lsts_amount_in: Vec<u128>, // 16 * 20
}
