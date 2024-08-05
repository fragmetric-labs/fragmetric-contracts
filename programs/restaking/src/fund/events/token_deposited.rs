use anchor_lang::prelude::*;

use super::FundInfo;

#[event]
pub struct FundTokenDeposited {
    pub user: Pubkey,
    pub user_lrt_account: Pubkey,

    pub deposited_token_mint: Pubkey,
    pub deposited_token_user_account: Pubkey,

    pub token_deposit_amount: u64,
    pub token_amount_in_fund: u128,
    pub minted_lrt_mint: Pubkey,
    pub minted_lrt_amount: u64,

    // Not Implemented Yet: Always `None` for now
    pub wallet_provider: Option<String>,
    pub fpoint_accrual_rate_multiplier: Option<f32>,

    pub fund_info: FundInfo,
}
