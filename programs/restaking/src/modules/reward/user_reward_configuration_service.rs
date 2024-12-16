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
            let mut user_reward_account = self.user_reward_account.load_init()?;

            // initialize account
            user_reward_account.initialize(
                user_reward_account_bump,
                self.receipt_token_mint,
                user_receipt_token_account,
            );

            // reflect existing token amount
            user_reward_account.update_user_reward_pools(
                &mut *self.reward_account.load_mut()?,
                Some(vec![TokenAllocatedAmountDelta::new_positive(
                    None,
                    user_receipt_token_account.amount,
                )]),
                self.current_slot,
            )?;

            return Ok(Some(events::UserCreatedOrUpdatedRewardAccount {
                receipt_token_mint: self.receipt_token_mint.key(),
                user_reward_account: self.user_reward_account.key(),
            }));
        }

        Ok(None)
    }

    pub fn process_update_user_reward_account_if_needed(
        &self,
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
            let mut user_reward_account = self.user_reward_account.load_mut()?;
            let initializing = user_reward_account.is_initializing();
            let updated = user_reward_account
                .update_if_needed(self.receipt_token_mint, user_receipt_token_account);

            // reflect existing token amount
            if initializing {
                user_reward_account.update_user_reward_pools(
                    &mut *self.reward_account.load_mut()?,
                    Some(vec![TokenAllocatedAmountDelta::new_positive(
                        None,
                        user_receipt_token_account.amount,
                    )]),
                    self.current_slot,
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
