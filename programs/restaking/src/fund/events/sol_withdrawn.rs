use anchor_lang::prelude::*;

use crate::fund::*;

#[event]
pub struct FundSOLWithdrawn {
    pub user: Pubkey,
    pub user_receipt: UserReceipt,

    pub request_id: u64,
    pub lrt_mint: Pubkey,
    pub lrt_amount: u64,
    pub lrt_price: u64,

    pub sol_withdraw_amount: u64,
    pub sol_fee_amount: u64,

    pub fund_info: FundInfo,
}
