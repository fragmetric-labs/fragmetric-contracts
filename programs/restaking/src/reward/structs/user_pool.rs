use anchor_lang::prelude::*;

use crate::{reward::*, PDASignerSeeds};

#[account]
#[derive(InitSpace)]
pub struct UserRewardAccount {
    pub data_version: u8,
    pub bump: u8,
    pub user: Pubkey,
    #[max_len(20)]
    pub user_reward_pools: Vec<UserRewardPool>,
}

impl PDASignerSeeds<3> for UserRewardAccount {
    const SEED: &'static [u8] = b"user_reward";

    fn signer_seeds(&self) -> [&[u8]; 3] {
        [Self::SEED, self.user.as_ref(), self.bump_as_slice()]
    }

    fn bump_ref(&self) -> &u8 {
        &self.bump
    }
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UserRewardPool {
    pub reward_pool_id: u8,
    pub token_allocated_amount: TokenAllocatedAmount,
    pub contribution: u128,
    pub updated_slot: u64,
    #[max_len(40)]
    pub reward_settlements: Vec<UserRewardSettlement>,
}

impl UserRewardPool {
    pub fn new(reward_pool_id: u8, reward_pool_initial_slot: u64) -> Self {
        Self {
            reward_pool_id,
            token_allocated_amount: Default::default(),
            contribution: 0,
            updated_slot: reward_pool_initial_slot,
            reward_settlements: vec![],
        }
    }
}
