use anchor_lang::prelude::*;

#[event]
pub struct UserClosedRewardAccount {
    pub receipt_token_mint: Pubkey,
    pub user: Pubkey,
    pub user_reward_account: Pubkey,
}
