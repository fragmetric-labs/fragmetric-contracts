use anchor_lang::prelude::*;
use crate::modules::reward::{RewardAccount, UserRewardAccount};

impl RewardAccount {
    pub fn initialize_if_needed(&mut self) {
        if self.data_version == 0 {
            self.data_version = 1;
        }
    }
}

impl UserRewardAccount {
    pub fn initialize_if_needed(&mut self, bump: u8, user: Pubkey) {
        if self.data_version == 0 {
            // version = 1 => lazily initialized by transfer hook
            // version = 2 => fully initialized by user
            self.data_version = 2;
            self.bump = bump;
            self.user = user;
        }
    }
}
