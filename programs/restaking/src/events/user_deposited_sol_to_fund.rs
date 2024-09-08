use anchor_lang::prelude::*;

use crate::modules::fund::{FundAccountInfo, UserFundAccount};

#[event]
pub struct UserDepositedSOLToFund {
    pub receipt_token_mint: Pubkey,
    pub fund_account: FundAccountInfo,

    pub user: Pubkey,
    pub user_receipt_token_account: Pubkey,
    pub user_fund_account: UserFundAccount,

    pub wallet_provider: Option<String>,
    pub contribution_accrual_rate: Option<u8>, // 100 is 1.0
    pub deposited_sol_amount: u64,
    pub minted_receipt_token_amount: u64,
}
