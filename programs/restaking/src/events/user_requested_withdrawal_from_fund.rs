use anchor_lang::prelude::*;

use crate::modules::fund::UserFundAccount;

#[event]
pub struct UserRequestedWithdrawalFromFund {
    pub receipt_token_mint: Pubkey,
    pub batch_id: u64,
    pub request_id: u64,

    pub user: Pubkey,
    pub user_receipt_token_account: Pubkey,
    pub user_fund_account: UserFundAccount,

    pub requested_receipt_token_amount: u64,
}
