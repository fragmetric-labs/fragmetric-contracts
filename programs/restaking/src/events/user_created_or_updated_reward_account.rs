use anchor_lang::prelude::*;

#[event]
pub struct UserCreatedOrUpdatedRewardAccount {
    pub receipt_token_mint: Pubkey,
    pub user_reward_account: Pubkey,
    pub receipt_token_amount: u64,
    pub created: bool,
}
