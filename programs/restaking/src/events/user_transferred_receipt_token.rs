use anchor_lang::prelude::*;
use crate::modules::fund::UserFundAccount;

#[event]
pub struct UserTransferredReceiptToken {
    pub receipt_token_mint: Pubkey,

    pub source_receipt_token_account: Pubkey,
    pub source_fund_account: UserFundAccount,
    pub source: Pubkey,

    pub destination_receipt_token_account: Pubkey,
    pub destination_fund_account: UserFundAccount,
    pub destination: Pubkey,

    pub transferred_receipt_token_amount: u64,
}
