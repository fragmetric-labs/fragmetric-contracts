use anchor_lang::prelude::*;

#[event]
pub struct OperatorClaimedRemainingReward {
    pub receipt_token_mint: Pubkey,
    pub reward_token_mint: Pubkey,
    pub program_revenue_account: Pubkey,
    pub program_reward_token_revenue_account: Pubkey,
    pub updated_reward_account: Pubkey,
    pub claimed_reward_token_amount: u64,
}
