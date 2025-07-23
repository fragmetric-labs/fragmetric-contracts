use anchor_lang::prelude::*;

#[event]
pub struct FundManagerDepositedToVault {
    pub vault: Pubkey,
    pub vault_supported_token_mint: Pubkey,
    pub vault_receipt_token_mint: Pubkey,

    pub deposited_vst_amount: u64,
    pub minted_vrt_amount: u64,
}
