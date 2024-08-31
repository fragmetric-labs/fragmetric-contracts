use anchor_lang::prelude::*;
use crate::modules::fund::{FundInfo, UserReceipt};

#[event]
pub struct UserDepositedTokenToFund {
    pub user: Pubkey,
    pub user_receipt_token_account: Pubkey,
    pub user_receipt: UserReceipt,

    pub supported_token_mint: Pubkey,
    pub supported_token_user_account: Pubkey,

    pub supported_token_deposit_amount: u64,
    pub minted_receipt_token_mint: Pubkey,
    pub minted_receipt_token_amount: u64,

    pub wallet_provider: Option<String>,
    pub contribution_accrual_rate: Option<f32>,

    pub fund_info: FundInfo,
}
