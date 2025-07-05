use anchor_lang::prelude::*;

#[event]
pub struct SolvManagerConfirmedDeposits {
    pub vault: Pubkey,
    pub solv_protocol_wallet: Pubkey,
    pub solv_manager: Pubkey,

    pub vst_mint: Pubkey,
    pub vrt_mint: Pubkey,
    pub srt_mint: Pubkey,

    pub solv_deposited_vst_amount: u64,
    pub srt_receivable_amount_for_deposit: u64,
    pub deposit_fee_as_vst: u64,
    pub one_srt_as_micro_vst: u64,
}
