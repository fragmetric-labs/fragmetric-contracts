use anchor_lang::prelude::*;

#[event]
pub struct SolvManagerCompletedWithdrawalRequests {
    pub vault: Pubkey,
    pub solv_protocol_wallet: Pubkey,
    pub solv_manager: Pubkey,

    pub vst_mint: Pubkey,
    pub vrt_mint: Pubkey,
    pub srt_mint: Pubkey,

    pub burnt_srt_amount: u64,
    pub withdrawan_vst_amount: u64,
    pub vst_reserved_amount_to_claim: u64,
    pub vst_extra_amount_to_claim: u64,
    pub vst_deducted_fee_amount: u64,
}
