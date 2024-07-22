use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct TokenInfo {
    pub address: Pubkey,
    pub token_cap: u64,
    pub token_amount_in: u128,
}
