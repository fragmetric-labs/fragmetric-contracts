use anchor_lang::prelude::*;

use crate::fund::*;

#[event]
pub struct FundWithdrawalRequested {
    pub user: Pubkey,
    pub user_lrt_account: Pubkey,
    pub user_receipt_account: Pubkey, // Receipt of withdrawal request

    pub request_id: u64,
    pub lrt_mint: Pubkey,
    pub lrt_requested_amount: u64,

    pub lrt_amount_in_user_account: u64,

    pub user_receipt: UserReceipt,
}
