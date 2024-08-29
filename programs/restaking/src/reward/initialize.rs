use anchor_lang::prelude::*;

use crate::reward::*;

impl RewardAccount {
    pub(super) fn initialize_if_needed(&mut self) {
        if self.data_version == 0 {
            self.data_version = 1;
            self.holders = vec![];
            self.rewards = vec![];
            self.reward_pools = vec![];
        }
    }
}

impl UserRewardAccount {
    pub(crate) fn initialize_if_needed(&mut self, bump: u8, user: Pubkey) {
        if self.data_version == 0 {
            self.data_version = 1;
            self.bump = bump;
            self.user = user;
            self.user_reward_pools = vec![];
        }
    }
}
