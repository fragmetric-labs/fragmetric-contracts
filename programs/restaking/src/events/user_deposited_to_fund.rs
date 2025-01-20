use anchor_lang::prelude::*;

#[event]
pub struct UserDepositedToFund {
    pub receipt_token_mint: Pubkey,
    pub fund_account: Pubkey,
    pub supported_token_mint: Option<Pubkey>,
    pub updated_user_reward_accounts: Vec<Pubkey>,

    pub user: Pubkey,
    pub user_receipt_token_account: Pubkey,
    pub user_fund_account: Pubkey,
    pub user_supported_token_account: Option<Pubkey>,

    pub wallet_provider: Option<String>,
    pub contribution_accrual_rate: Option<u16>, // 100 is 1.0
    pub deposited_amount: u64,
    pub minted_receipt_token_amount: u64,
}
