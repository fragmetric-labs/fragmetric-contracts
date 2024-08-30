use anchor_lang::prelude::*;

use crate::reward::*;

#[event]
pub struct UserUpdatedRewardPool {
    pub updates: Vec<UserRewardAccountUpdatedInfo>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UserRewardAccountUpdatedInfo {
    pub user: Pubkey,
    pub updated_user_reward_pools: Vec<UserRewardPool>,
}

impl UserUpdatedRewardPool {
    pub fn new_from_updates(
        from_user_update: Option<UserRewardAccountUpdatedInfo>,
        to_user_update: Option<UserRewardAccountUpdatedInfo>,
    ) -> Self {
        let updates = from_user_update
            .into_iter()
            .chain(to_user_update)
            .collect();
        Self { updates }
    }
}

impl UserRewardAccountUpdatedInfo {
    pub fn new_from_user_reward_pool(user: Pubkey, user_reward_pool: Vec<UserRewardPool>) -> Self {
        Self {
            user,
            updated_user_reward_pools: user_reward_pool,
        }
    }
}
