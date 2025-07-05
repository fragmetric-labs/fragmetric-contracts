use anchor_lang::prelude::*;

#[event]
pub struct SolvManagerConfirmedWithdrawalRequests {
    pub vault: Pubkey,
    pub solv_protocol_wallet: Pubkey,
    pub solv_manager: Pubkey,

    pub vst_mint: Pubkey,
    pub vrt_mint: Pubkey,
    pub srt_mint: Pubkey,

    pub confirmed_srt_amount: u64,
    pub processing_vrt_amount: u64,
    pub vst_receivable_amount_to_claim: u64,
}
