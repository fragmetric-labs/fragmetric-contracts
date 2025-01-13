use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::events;
use crate::utils::{AccountInfoExt, AccountLoaderExt, AsAccountInfo, PDASeeds, SystemProgramExt};

use super::*;

pub struct UserRewardConfigurationService<'info: 'a, 'a> {
    receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
    user: &'a Signer<'info>,
    reward_account: &'a mut AccountLoader<'info, RewardAccount>,
    user_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,
    _current_slot: u64,
}

impl<'info, 'a> UserRewardConfigurationService<'info, 'a> {
    pub fn process_create_user_reward_account_idempotent<'b>(
        system_program: &'b Program<'info, System>,
        receipt_token_mint: &'b mut InterfaceAccount<'info, Mint>,
        reward_account: &'b mut AccountLoader<'info, RewardAccount>,

        user: &'b Signer<'info>,
        user_receipt_token_account: &'b InterfaceAccount<'info, TokenAccount>,
        user_reward_account: &'b mut UncheckedAccount<'info>,
        user_reward_account_bump: u8,

        desired_account_size: Option<u32>,
    ) -> Result<Option<events::UserCreatedOrUpdatedRewardAccount>> {
        if !user_reward_account.is_initialized() {
            system_program.initialize_account(
                user_reward_account,
                user,
                &[&[
                    UserRewardAccount::SEED,
                    receipt_token_mint.key().as_ref(),
                    user.key().as_ref(),
                    &[user_reward_account_bump],
                ]],
                std::cmp::min(
                    solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE,
                    8 + std::mem::size_of::<UserRewardAccount>(),
                ),
                None,
                &crate::ID,
            )?;

            let mut user_reward_account_parsed =
                AccountLoader::<UserRewardAccount>::try_from_unchecked(
                    &crate::ID,
                    user_reward_account.as_account_info(),
                )?;

            UserRewardConfigurationService::new(
                receipt_token_mint,
                user,
                reward_account,
                &mut user_reward_account_parsed,
            )?
            .process_initialize_user_reward_account(
                user_receipt_token_account,
                user_reward_account_bump,
            )
        } else {
            let mut user_reward_account_parsed = AccountLoader::<UserRewardAccount>::try_from(
                user_reward_account.as_account_info(),
            )?;

            require_eq!(
                user_reward_account_bump,
                user_reward_account_parsed.load()?.get_bump()
            );

            UserRewardConfigurationService::new(
                receipt_token_mint,
                user,
                reward_account,
                &mut user_reward_account_parsed,
            )?
            .process_update_user_reward_account_if_needed(
                user_receipt_token_account,
                system_program,
                desired_account_size,
            )
        }
    }

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
            _current_slot: Clock::get()?.slot,
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
        let min_account_size = 8 + std::mem::size_of::<UserRewardAccount>();
        let target_account_size = desired_account_size
            .map(|size| std::cmp::max(size as usize, min_account_size))
            .unwrap_or(min_account_size);

        let new_account_size = system_program.expand_account_size_if_needed(
            self.user_reward_account.as_ref(),
            self.user,
            &[],
            target_account_size,
            None,
        )?;

        if new_account_size >= min_account_size {
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
