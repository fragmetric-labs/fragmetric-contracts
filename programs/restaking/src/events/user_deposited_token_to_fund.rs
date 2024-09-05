use anchor_lang::prelude::*;

use crate::modules::fund::{FundAccountInfo, UserFundAccount};

#[event]
pub struct UserDepositedTokenToFund {
    pub receipt_token_mint: Pubkey,
    pub fund_account: FundAccountInfo,

    pub user: Pubkey,
    pub user_receipt_token_account: Pubkey,
    pub user_fund_account: UserFundAccount,

    pub supported_token_mint: Pubkey,
    pub supported_token_user_account: Pubkey,

    pub wallet_provider: Option<String>,
    pub contribution_accrual_rate: Option<f32>,
    pub deposited_supported_token_amount: u64,
    pub minted_receipt_token_amount: u64,
}
