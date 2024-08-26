use anchor_lang::prelude::*;

#[event]
pub struct ReceiptTokenTransferred {
    pub receipt_token_mint: Pubkey,
    pub receipt_token_amount: u64,
    pub source_receipt_token_account: Pubkey,
    pub source_receipt_token_account_owner: Pubkey,
    pub destination_receipt_token_account: Pubkey,
    pub destination_receipt_token_account_onwer: Pubkey,
}
