use anchor_lang::prelude::*;
use crate::modules::fund::{FundInfo, UserReceipt};

#[event]
pub struct UserDepositedSOLToFund {
    pub user: Pubkey,
    pub user_receipt_token_account: Pubkey,
    pub user_receipt: UserReceipt,

    pub sol_deposit_amount: u64,
    pub minted_receipt_token_mint: Pubkey,
    pub minted_receipt_token_amount: u64,

    pub wallet_provider: Option<String>,
    pub contribution_accrual_rate: Option<f32>,

    pub fund_info: FundInfo,
}
