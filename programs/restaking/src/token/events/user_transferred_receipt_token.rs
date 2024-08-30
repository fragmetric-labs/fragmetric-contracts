use anchor_lang::prelude::*;

use crate::UserReceipt;

#[event]
pub struct UserTransferredReceiptToken {
    pub transferred_receipt_token_mint: Pubkey,
    pub transferred_receipt_token_amount: u64,
    pub source_receipt_token_account: Pubkey,
    pub source_user: Pubkey,
    pub source_user_receipt: UserReceipt,
    pub destination_receipt_token_account: Pubkey,
    pub destination_user: Pubkey,
    pub destination_user_receipt: UserReceipt,
}
