use anchor_lang::prelude::*;

#[event]
pub struct SolvManagerCompletedDeposits {
    pub vault: Pubkey,
    pub solv_protocol_wallet: Pubkey,
    pub solv_manager: Pubkey,

    pub vst_mint: Pubkey,
    pub vrt_mint: Pubkey,
    pub srt_mint: Pubkey,

    pub received_srt_amount: u64,
    pub srt_total_reserved_amount: u64,
    pub old_one_srt_as_micro_vst: u64,
    pub new_one_srt_as_micro_vst: u64,
    pub extra_deposit_fee_as_vst: u64,
}
