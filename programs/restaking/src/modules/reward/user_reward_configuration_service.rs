use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::events;
use crate::utils::AccountLoaderExt;

use super::*;

pub struct UserRewardConfigurationService<'info: 'a, 'a> {
    receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
    user: &'a Signer<'info>,
    user_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,
}

impl<'info, 'a> UserRewardConfigurationService<'info, 'a> {
    pub fn new(
        receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
        user: &'a Signer<'info>,
        user_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,
    ) -> Result<Self> {
        Ok(Self {
            receipt_token_mint,
            user,
            user_reward_account,
        })
    }

    pub fn process_initialize_user_reward_account(
        &mut self,
        user_reward_account_bump: u8,
    ) -> Result<()> {
        if self.user_reward_account.as_ref().data_len()
            < 8 + std::mem::size_of::<UserRewardAccount>()
        {
            self.user_reward_account
                .initialize_zero_copy_header(user_reward_account_bump)?;
        } else {
            self.user_reward_account.load_init()?.initialize(
                user_reward_account_bump,
                self.receipt_token_mint.key(),
                self.user.key(),
            );
        }
        Ok(())
    }

    pub fn process_update_user_reward_account_if_needed(
        &self,
        system_program: &Program<'info, System>,
        desired_account_size: Option<u32>,
    ) -> Result<()> {
        self.user_reward_account.expand_account_size_if_needed(
            self.user,
            system_program,
            desired_account_size,
        )?;

        if self.user_reward_account.as_ref().data_len()
            >= 8 + std::mem::size_of::<UserRewardAccount>()
        {
            self.user_reward_account
                .load_mut()?
                .update_if_needed(self.receipt_token_mint.key(), self.user.key());
        }

        Ok(())
    }
}
