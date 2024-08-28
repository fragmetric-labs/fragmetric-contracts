use anchor_lang::prelude::*;

use crate::fund::*;

#[event]
pub struct UserRequestedWithdrawalFromFund {
    pub user: Pubkey,
    pub user_receipt_token_account: Pubkey,
    pub user_receipt: UserReceipt,

    pub batch_id: u64,
    pub request_id: u64,
    pub requested_receipt_token_mint: Pubkey,
    pub requested_receipt_token_amount: u64,
}
