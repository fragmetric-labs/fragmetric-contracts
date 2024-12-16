use anchor_lang::prelude::*;

#[event]
pub struct UserCanceledWithdrawalRequestFromFund {
    pub receipt_token_mint: Pubkey,
    pub fund_account: Pubkey,
    pub supported_token_mint: Option<Pubkey>,
    pub updated_user_reward_accounts: Vec<Pubkey>,

    pub user: Pubkey,
    pub user_receipt_token_account: Pubkey,
    pub user_fund_account: Pubkey,

    pub batch_id: u64,
    pub request_id: u64,
    pub requested_receipt_token_amount: u64,
}
