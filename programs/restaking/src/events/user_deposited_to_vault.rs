use anchor_lang::prelude::*;

#[event]
// This event's name is quite inaccurate, but named like this because this emits when user deposits his/her vault receipt token which received from the 'vault' to the fund.
// So it's named like this for consistency with UserDepositedToFund event.
pub struct UserDepositedToVault {
    pub receipt_token_mint: Pubkey,
    pub fund_account: Pubkey,
    pub vault_account: Pubkey,
    pub vault_receipt_token_mint: Pubkey,
    pub updated_user_reward_accounts: Vec<Pubkey>,

    pub user: Pubkey,
    pub user_receipt_token_account: Pubkey,
    pub user_fund_account: Pubkey,
    pub user_vault_receipt_token_account: Pubkey,

    pub wallet_provider: Option<String>,
    pub contribution_accrual_rate: Option<u16>, // 100 is 1.0
    pub deposited_amount: u64,
    pub minted_receipt_token_amount: u64,
}
