use anchor_lang::prelude::*;

#[event]
pub struct FundManagerWithdrewFromVault {
    pub vault: Pubkey,
    pub vault_supported_token_mint: Pubkey,
    pub vault_receipt_token_mint: Pubkey,

    pub burnt_vrt_amount: u64,
    /// Estimated amount minus fee
    pub claimed_vst_amount: u64,
    /// Extra claimed amount if exists
    pub extra_vst_amount: u64,
    /// Withdrawal fee
    pub deducted_vst_fee_amount: u64,
}
