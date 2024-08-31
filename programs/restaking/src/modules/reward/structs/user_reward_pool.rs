use anchor_lang::prelude::*;

use crate::modules::common::PDASignerSeeds;
use crate::modules::reward::{TokenAllocatedAmount, UserRewardSettlement, REWARD_POOLS_MAX_LEN, REWARDS_MAX_LEN};

#[account]
#[derive(InitSpace)]
pub struct UserRewardAccount {
    pub data_version: u8,
    pub bump: u8,
    pub user: Pubkey,
    #[max_len(REWARD_POOLS_MAX_LEN)]
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
    pub _reserved: [u8; 64],
    #[max_len(REWARDS_MAX_LEN)]
    pub reward_settlements: Vec<UserRewardSettlement>,
}

impl UserRewardPool {
    pub fn new(reward_pool_id: u8, reward_pool_initial_slot: u64) -> Self {
        Self {
            reward_pool_id,
            token_allocated_amount: Default::default(),
            contribution: 0,
            updated_slot: reward_pool_initial_slot,
            _reserved: [0; 64],
            reward_settlements: vec![],
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UserRewardAccountUpdateInfo {
    pub user: Pubkey,
    pub updated_user_reward_pools: Vec<UserRewardPool>,
}

impl UserRewardAccountUpdateInfo {
    pub fn new_from_user_reward_pool(user: Pubkey, user_reward_pool: Vec<UserRewardPool>) -> Self {
        Self {
            user,
            updated_user_reward_pools: user_reward_pool,
        }
    }
}
