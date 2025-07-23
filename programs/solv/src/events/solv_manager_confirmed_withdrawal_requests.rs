use anchor_lang::prelude::*;

#[event]
pub struct SolvManagerConfirmedWithdrawalRequests {
    pub vault: Pubkey,
    pub solv_protocol_wallet: Pubkey,
    pub solv_manager: Pubkey,

    pub vault_supported_token_mint: Pubkey,
    pub vault_receipt_token_mint: Pubkey,
    pub solv_receipt_token_mint: Pubkey,

    pub confirmed_srt_amount: u64,
    pub estimated_vst_amount: u64,
    pub one_srt_as_micro_vst: u64,
}
