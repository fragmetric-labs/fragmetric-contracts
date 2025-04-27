use anchor_lang::prelude::*;

#[event]
pub struct UserCreatedOrUpdatedFundAccount {
    pub receipt_token_mint: Pubkey,
    pub user: Pubkey,
    pub user_fund_account: Pubkey,
    pub receipt_token_amount: u64,
    pub created: bool,
}
