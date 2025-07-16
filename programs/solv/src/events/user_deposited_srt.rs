use anchor_lang::prelude::*;

#[event]
pub struct UserDepositedSRT {
    pub vault: Pubkey,
    pub vault_receipt_token_mint: Pubkey,

    pub user: Pubkey,
    pub user_vrt_account: Pubkey,

    pub deposited_srt_amount: u64,
    pub minted_vrt_amount: u64,
}
