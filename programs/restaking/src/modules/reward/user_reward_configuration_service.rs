use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::events;
use crate::utils::AccountLoaderExt;

use super::*;

pub struct UserRewardConfigurationService<'info: 'a, 'a> {
    receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
    user: &'a Signer<'info>,
    reward_account: &'a mut AccountLoader<'info, RewardAccount>,
    user_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,
    current_slot: u64,
}

impl<'info, 'a> UserRewardConfigurationService<'info, 'a> {
    pub fn new(
        receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
        user: &'a Signer<'info>,
        reward_account: &'a mut AccountLoader<'info, RewardAccount>,
        user_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,
    ) -> Result<Self> {
        Ok(Self {
            receipt_token_mint,
            user,
            reward_account,
            user_reward_account,
            current_slot: Clock::get()?.slot,
        })
    }

    pub fn process_initialize_user_reward_account(
        &mut self,
        user_receipt_token_account: &InterfaceAccount<'info, TokenAccount>,
        user_reward_account_bump: u8,
    ) -> Result<Option<events::UserCreatedOrUpdatedRewardAccount>> {
        if self.user_reward_account.as_ref().data_len()
            < 8 + std::mem::size_of::<UserRewardAccount>()
        {
            self.user_reward_account
                .initialize_zero_copy_header(user_reward_account_bump)?;
        } else {
            // initialize account
            self.user_reward_account.load_init()?.initialize(
                user_reward_account_bump,
                self.receipt_token_mint,
                user_receipt_token_account,
            );
            self.user_reward_account.exit(&crate::ID)?;

            // reflect existing token amount
            RewardService::new(self.receipt_token_mint, self.reward_account)?
                .update_reward_pools_token_allocation(
                    None,
                    Some(self.user_reward_account),
                    user_receipt_token_account.amount,
                    None,
                )?;

            return Ok(Some(events::UserCreatedOrUpdatedRewardAccount {
                receipt_token_mint: self.receipt_token_mint.key(),
                user_reward_account: self.user_reward_account.key(),
            }));
        }

        Ok(None)
    }

    pub fn process_update_user_reward_account_if_needed(
        &mut self,
        user_receipt_token_account: &InterfaceAccount<'info, TokenAccount>,
        system_program: &Program<'info, System>,
        desired_account_size: Option<u32>,
    ) -> Result<Option<events::UserCreatedOrUpdatedRewardAccount>> {
        self.user_reward_account.expand_account_size_if_needed(
            self.user,
            system_program,
            desired_account_size,
        )?;

        if self.user_reward_account.as_ref().data_len()
            >= 8 + std::mem::size_of::<UserRewardAccount>()
        {
            let (initializing, updated) = {
                let mut user_reward_account = self.user_reward_account.load_mut()?;
                let initializing = user_reward_account.is_initializing();
                let updated = user_reward_account
                    .update_if_needed(self.receipt_token_mint, user_receipt_token_account);
                (initializing, updated)
            };

            // reflect existing token amount
            if initializing {
                RewardService::new(self.receipt_token_mint, self.reward_account)?
                    .update_reward_pools_token_allocation(
                        None,
                        Some(self.user_reward_account),
                        user_receipt_token_account.amount,
                        None,
                    )?;
            }

            if initializing || updated {
                return Ok(Some(events::UserCreatedOrUpdatedRewardAccount {
                    receipt_token_mint: self.receipt_token_mint.key(),
                    user_reward_account: self.user_reward_account.key(),
                }));
            }
        }
        Ok(None)
    }
}
