use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Fund {
    pub admin: Pubkey,                  // 32
    pub default_protocol_fee_rate: u16, // 2
    pub receipt_token_mint: Pubkey,     // 32
    #[max_len(20)]
    pub whitelisted_tokens: Vec<TokenInfo>,
    // pub receipt_token_lock_account: Pubkey, // 32
    pub sol_amount_in: u128, // 16
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct TokenInfo {
    pub address: Pubkey,
    pub token_cap: u128,
    pub token_amount_in: u128,
}

impl TokenInfo {
    pub fn empty(address: Pubkey, token_cap: u128) -> Self {
        Self {
            address,
            token_cap,
            token_amount_in: 0,
        }
    }
}
