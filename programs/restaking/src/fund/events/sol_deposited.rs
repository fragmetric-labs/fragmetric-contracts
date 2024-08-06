use anchor_lang::prelude::*;

use crate::fund::*;

#[event]
pub struct FundSOLDeposited {
    pub user: Pubkey,
    pub user_lrt_account: Pubkey,

    pub sol_deposit_amount: u64,
    pub sol_amount_in_fund: u128,
    pub minted_lrt_mint: Pubkey,
    pub minted_lrt_amount: u64,
    pub lrt_amount_in_user_lrt_account: u64,

    // Not Implemented Yet: Always `None` for now
    pub wallet_provider: Option<String>,
    pub fpoint_accrual_rate_multiplier: Option<f32>,

    pub fund_info: FundInfo,
    pub user_receipt: UserReceipt,
}
