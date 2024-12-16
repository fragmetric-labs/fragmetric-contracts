use super::*;
use crate::events;
use anchor_lang::prelude::*;

pub struct UserRewardService<'info: 'a, 'a> {
    reward_account: &'a mut AccountLoader<'info, RewardAccount>,
    user_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,

    current_slot: u64,
}

impl<'info, 'a> UserRewardService<'info, 'a> {
    pub fn new(
        reward_account: &'a mut AccountLoader<'info, RewardAccount>,
        user_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            reward_account,
            user_reward_account,
            current_slot: clock.slot,
        })
    }

    pub fn process_update_user_reward_pools(&self) -> Result<()> {
        self.user_reward_account
            .load_mut()?
            .update_user_reward_pools(&mut *self.reward_account.load_mut()?, self.current_slot)
    }

    pub fn process_claim_user_rewards(&self) -> Result<()> {
        unimplemented!()
    }
}
