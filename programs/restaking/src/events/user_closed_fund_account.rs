use anchor_lang::prelude::*;

#[event]
pub struct UserClosedFundAccount {
    pub receipt_token_mint: Pubkey,
    pub user: Pubkey,
    pub user_fund_account: Pubkey,
}
