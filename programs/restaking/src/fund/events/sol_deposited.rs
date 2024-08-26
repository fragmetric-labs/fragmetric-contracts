use anchor_lang::prelude::*;

use crate::fund::*;

#[event]
pub struct FundSOLDeposited {
    pub user: Pubkey,
    pub user_receipt_token_account: Pubkey,
    pub user_receipt: UserReceipt,

    pub sol_deposit_amount: u64,
    pub sol_amount_in_fund: u64,
    pub minted_receipt_token_mint: Pubkey,
    pub minted_receipt_token_amount: u64,
    pub receipt_token_price: u64,
    pub receipt_token_amount_in_user_receipt_token_account: u64,

    // Not Implemented Yet: Always `None` for now
    pub wallet_provider: Option<String>,
    pub fpoint_accrual_rate_multiplier: Option<f32>,

    pub fund_info: FundInfo,
}
