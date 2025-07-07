use anchor_lang::prelude::*;

#[event]
pub struct SolvManagerImpliedSolvProtocolFee {
    pub vault: Pubkey,
    pub solv_protocol_wallet: Pubkey,
    pub solv_manager: Pubkey,

    pub vault_supported_token_mint: Pubkey,
    pub vault_receipt_token_mint: Pubkey,
    pub solv_receipt_token_mint: Pubkey,

    pub old_one_srt_as_micro_vst: u64,
    pub new_one_srt_as_micro_vst: u64,
    pub srt_operation_reserved_amount_as_vst_delta: u64,
    pub vst_operation_receivable_amount: u64,
}
