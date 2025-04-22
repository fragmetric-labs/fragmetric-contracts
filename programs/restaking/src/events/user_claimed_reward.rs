use anchor_lang::prelude::*;

#[event]
pub struct UserClaimedReward {
    pub receipt_token_mint: Pubkey,
    pub reward_token_mint: Pubkey,
    pub destination_reward_token_account: Pubkey,
    pub destination_reward_token_account_owner: Pubkey,
    pub updated_reward_account: Pubkey,
    pub updated_user_reward_account: Pubkey,
    pub claimed_reward_token_amount: u64,
    pub total_claimed_reward_token_amount: u64,
}
