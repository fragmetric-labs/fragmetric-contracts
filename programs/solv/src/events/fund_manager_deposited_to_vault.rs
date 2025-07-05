use anchor_lang::prelude::*;

#[event]
pub struct FundManagerDepositedToVault {
    pub vault: Pubkey,
    pub vst_mint: Pubkey,
    pub vrt_mint: Pubkey,

    pub vst_amount: u64,
    pub minted_vrt_amount: u64,
}
