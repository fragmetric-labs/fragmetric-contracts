use anchor_lang::prelude::*;

#[event]
pub struct FundManagerRequestedWithdrawalFromVault {
    pub vault: Pubkey,
    pub vst_mint: Pubkey,
    pub vrt_mint: Pubkey,

    pub burnt_vrt_amount: u64,
    pub vst_estimated_amount: u64,
}
