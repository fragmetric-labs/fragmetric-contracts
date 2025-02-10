use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::events;

use super::*;

pub struct UserRewardService<'info: 'a, 'a> {
    receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
    reward_account: &'a mut AccountLoader<'info, RewardAccount>,
    user_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,
    current_slot: u64,
}

impl<'info, 'a> UserRewardService<'info, 'a> {
    pub fn new(
        receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
        user: &Signer,
        reward_account: &'a mut AccountLoader<'info, RewardAccount>,
        user_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,
    ) -> Result<Self> {
        require_keys_eq!(user_reward_account.load()?.user, user.key());

        let clock = Clock::get()?;
        Ok(Self {
            receipt_token_mint,
            reward_account,
            user_reward_account,
            current_slot: clock.slot,
        })
    }

    /// Allow off-curve users
    pub fn new_with_user_seeds(
        receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
        user: &AccountInfo,
        user_signer_seeds: &[&[u8]],
        reward_account: &'a mut AccountLoader<'info, RewardAccount>,
        user_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,
    ) -> Result<Self> {
        require_keys_eq!(
            user.key(),
            Pubkey::create_program_address(user_signer_seeds, &crate::ID)
                .map_err(|_| ProgramError::InvalidSeeds)?,
        );
        require_keys_eq!(user_reward_account.load()?.user, user.key());

        let clock = Clock::get()?;
        Ok(Self {
            receipt_token_mint,
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
