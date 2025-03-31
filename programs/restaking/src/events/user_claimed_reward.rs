use anchor_lang::prelude::*;

#[event]
pub struct UserClaimedReward {
    pub receipt_token_mint: Pubkey,
    pub reward_token_mint: Pubkey,
    pub reward_account: Pubkey,
    pub user_reward_account: Pubkey,
    pub claimed_reward_token_amount: u64,
    pub total_claimed_reward_token_amount: u64,
}
