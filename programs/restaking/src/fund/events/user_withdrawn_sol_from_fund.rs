use anchor_lang::prelude::*;

use crate::fund::*;

#[event]
pub struct UserWithdrawnSOLFromFund {
    pub user: Pubkey,
    pub user_receipt: UserReceipt,

    pub request_id: u64,
    pub receipt_token_mint: Pubkey,
    pub receipt_token_amount: u64,

    pub sol_withdraw_amount: u64,
    pub sol_fee_amount: u64,

    pub fund_info: FundInfo,
}
