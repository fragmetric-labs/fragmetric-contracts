use anchor_lang::prelude::*;

#[event]
pub struct UserWrappedReceiptToken {
    pub receipt_token_mint: Pubkey,
    pub wrapped_token_mint: Pubkey,
    pub fund_account: Pubkey,

    pub user: Pubkey,
    pub user_receipt_token_account: Pubkey,
    pub user_wrapped_token_account: Pubkey,
    pub user_fund_account: Option<Pubkey>,
    pub user_reward_account: Option<Pubkey>,

    pub wrapped_receipt_token_amount: u64,
}
