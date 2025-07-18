use anchor_lang::prelude::*;

#[event]
pub struct UserDepositedSolvReceiptTokenToVault {
    pub vault: Pubkey,
    pub solv_receipt_token_mint: Pubkey,
    pub vault_receipt_token_mint: Pubkey,

    pub operation_reserved_srt_amount: u64,
    pub one_srt_as_micro_vst: u64,
    pub vrt_supply: u64,
    pub one_vrt_as_micro_vst: u64,

    pub user: Pubkey,
    pub deposited_srt_amount: u64,
    pub minted_vrt_amount: u64,
}
