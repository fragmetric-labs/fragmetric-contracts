use anchor_lang::prelude::*;

#[event]
pub struct TokenLRTTransferred {
    pub lrt_mint: Pubkey,
    pub lrt_amount: u64,
    pub source_lrt_account: Pubkey,
    pub source_lrt_account_owner: Pubkey,
    pub destination_lrt_account: Pubkey,
    pub destination_lrt_account_onwer: Pubkey,
}
