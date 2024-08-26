use anchor_lang::prelude::*;

use crate::fund::*;

#[event]
pub struct FundWithdrawalRequestCanceled {
    pub user: Pubkey,
    pub user_receipt_token_account: Pubkey,
    pub user_receipt: UserReceipt,

    pub request_id: u64,
    pub receipt_token_mint: Pubkey,
    pub receipt_token_requested_amount: u64,

    pub receipt_token_amount_in_user_receipt_token_account: u64,
}
