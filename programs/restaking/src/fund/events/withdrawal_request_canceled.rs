use anchor_lang::prelude::*;

use crate::fund::*;

#[event]
pub struct FundWithdrawalRequestCanceled {
    pub user: Pubkey,
    pub user_lrt_account: Pubkey,
    pub user_receipt: UserReceipt,

    pub request_id: u64,
    pub lrt_mint: Pubkey,
    pub lrt_requested_amount: u64,

    pub lrt_amount_in_user_lrt_account: u64,
}
