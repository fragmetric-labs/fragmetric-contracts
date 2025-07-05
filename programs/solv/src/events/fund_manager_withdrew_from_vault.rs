use anchor_lang::prelude::*;

#[event]
pub struct FundManagerWithdrewFromVault {
    pub vault: Pubkey,
    pub vst_mint: Pubkey,
    pub vrt_mint: Pubkey,

    pub claimed_vst_amount: u64,
}
