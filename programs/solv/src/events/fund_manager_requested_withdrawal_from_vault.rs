use anchor_lang::prelude::*;

#[event]
pub struct FundManagerRequestedWithdrawalFromVault {
    pub vault: Pubkey,
    pub vault_supported_token_mint: Pubkey,
    pub vault_receipt_token_mint: Pubkey,

    pub requested_vrt_amount: u64,
    pub estimated_vst_withdrawal_amount: u64,
}
