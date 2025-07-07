use anchor_lang::prelude::*;

#[event]
pub struct FundManagerWithdrewFromVault {
    pub vault: Pubkey,
    pub vault_supported_token_mint: Pubkey,
    pub vault_receipt_token_mint: Pubkey,

    pub claimed_vst_amount: u64,
}
