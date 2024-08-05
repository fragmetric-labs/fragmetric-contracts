use anchor_lang::prelude::*;

use crate::fund::*;

#[event]
pub struct FundSOLWithdrawed {
    pub user: Pubkey,
    pub user_receipt_account: Pubkey, // Receipt of withdrawal request

    pub request_id: u64,
    pub lrt_mint: Pubkey,
    pub lrt_amount: u64,

    pub sol_withdraw_amount: u64,
    pub sol_fee_amount: u64,

    pub fund_info: FundInfo,
    pub user_receipt: UserReceipt,
}
