use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::events;

use super::*;

pub struct UserRewardService<'info: 'a, 'a> {
    receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
    user: &'a Signer<'info>,
    reward_account: &'a mut AccountLoader<'info, RewardAccount>,
    user_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,
    current_slot: u64,
}

impl<'info, 'a> UserRewardService<'info, 'a> {
    pub fn new(
        receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
        user: &'a Signer<'info>,
        reward_account: &'a mut AccountLoader<'info, RewardAccount>,
        user_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            receipt_token_mint,
            user,
            reward_account,
            user_reward_account,
            current_slot: clock.slot,
        })
    }

    pub fn process_update_user_reward_pools(&self) -> Result<events::UserUpdatedRewardPool> {
        self.user_reward_account
            .load_mut()?
            .update_user_reward_pools(&mut *self.reward_account.load_mut()?, self.current_slot)?;

        Ok(events::UserUpdatedRewardPool {
            receipt_token_mint: self.receipt_token_mint.key(),
            updated_user_reward_accounts: vec![self.user_reward_account.key()],
        })
    }

    pub fn process_claim_user_rewards(&self) -> Result<()> {
        unimplemented!()
    }
}
