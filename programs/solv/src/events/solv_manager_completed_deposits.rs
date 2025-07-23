use anchor_lang::prelude::*;

#[event]
pub struct SolvManagerCompletedDeposits {
    pub vault: Pubkey,
    pub solv_protocol_wallet: Pubkey,
    pub solv_manager: Pubkey,

    pub vault_supported_token_mint: Pubkey,
    pub vault_receipt_token_mint: Pubkey,
    pub solv_receipt_token_mint: Pubkey,

    pub received_srt_amount: u64,
    pub operation_reserved_srt_amount: u64,
    pub deducted_vst_extra_fee_amount: u64,
    pub old_one_srt_as_micro_vst: u64,
    pub new_one_srt_as_micro_vst: u64,
    pub old_one_vrt_as_micro_vst: u64,
    pub new_one_vrt_as_micro_vst: u64,
}
