use anchor_lang::prelude::*;

use crate::{constants::*, reward::*};

#[event]
pub struct AdminUpdatedRewardPool {
    pub address: Pubkey,
    pub holders: Vec<Holder>,
    pub rewards: Vec<Reward>,
    pub updated_reward_pool_ids: Vec<u8>,
}

impl AdminUpdatedRewardPool {
    pub fn new_from_reward_account(
        reward_account: &RewardAccount,
        updated_reward_pool_ids: Vec<u8>,
    ) -> Self {
        Self {
            address: REWARD_ACCOUNT_ADDRESS,
            holders: reward_account.holders.clone(),
            rewards: reward_account.rewards.clone(),
            updated_reward_pool_ids,
        }
    }
}
