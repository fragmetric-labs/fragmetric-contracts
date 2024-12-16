use anchor_lang::prelude::*;

use crate::modules::fund::{FundAccountInfo, UserFundAccount};

#[event]
pub struct UserWithdrewFromFund {
    pub receipt_token_mint: Pubkey,
    pub request_id: u64,

    pub user_fund_account: UserFundAccount,
    pub user: Pubkey,

    pub burnt_receipt_token_amount: u64,
    pub supported_token_mint: Option<Pubkey>,
    pub withdrawn_amount: u64,
    pub deducted_fee_amount: u64,

    pub fund_account: FundAccountInfo,
}
