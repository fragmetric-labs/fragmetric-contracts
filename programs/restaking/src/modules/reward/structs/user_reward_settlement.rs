use anchor_lang::prelude::*;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UserRewardSettlement {
    pub reward_id: u8,
    pub settled_amount: u64,
    pub settled_contribution: u128,
    pub settled_slot: u64,
    pub claimed_amount: u64,
}

impl UserRewardSettlement {
    pub fn new(reward_id: u8, reward_pool_initial_slot: u64) -> Self {
        Self {
            reward_id,
            settled_amount: 0,
            claimed_amount: 0,
            settled_contribution: 0,
            settled_slot: reward_pool_initial_slot,
        }
    }
}