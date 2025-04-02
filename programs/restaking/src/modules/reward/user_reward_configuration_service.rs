use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::events;
use crate::utils::{AccountInfoExt, AccountLoaderExt, AsAccountInfo, PDASeeds, SystemProgramExt};

use super::*;

pub struct UserRewardConfigurationService<'a, 'info> {
    receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
    user_receipt_token_account: &'a InterfaceAccount<'info, TokenAccount>,
    reward_account: &'a mut AccountLoader<'info, RewardAccount>,
    user_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,
    _current_slot: u64,
}

impl Drop for UserRewardConfigurationService<'_, '_> {
    fn drop(&mut self) {
        self.reward_account.exit(&crate::ID).unwrap();
        self.user_reward_account.exit(&crate::ID).unwrap();
    }
}

impl<'a, 'info> UserRewardConfigurationService<'a, 'info> {
    pub fn process_create_user_reward_account_idempotent(
        system_program: &Program<'info, System>,
        receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
        reward_account: &mut AccountLoader<'info, RewardAccount>,

        payer: &Signer<'info>,
        user_receipt_token_account: &InterfaceAccount<'info, TokenAccount>,
        user_reward_account: &mut UncheckedAccount<'info>,
        user_reward_account_bump: u8,

        desired_account_size: Option<u32>,
    ) -> Result<Option<events::UserCreatedOrUpdatedRewardAccount>> {
        if !user_reward_account.is_initialized() {
            system_program.initialize_account(
                user_reward_account,
                payer,
                &[&[
                    UserRewardAccount::SEED,
                    receipt_token_mint.key().as_ref(),
                    user_receipt_token_account.owner.as_ref(),
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

            let event = UserRewardConfigurationService::new(
                receipt_token_mint,
                user_receipt_token_account,
                reward_account,
                &mut user_reward_account_parsed,
            )?
            .process_initialize_user_reward_account(user_reward_account_bump)?;

            Ok(event)
        } else {
            let mut user_reward_account_parsed = AccountLoader::<UserRewardAccount>::try_from(
                user_reward_account.as_account_info(),
            )?;

            // Constraint check
            // bump = user_reward_account.get_bump()?
            require_eq!(
                user_reward_account_bump,
                user_reward_account_parsed.get_bump()?,
            );

            let event = UserRewardConfigurationService::new(
                receipt_token_mint,
                user_receipt_token_account,
                reward_account,
                &mut user_reward_account_parsed,
            )?
            .process_update_user_reward_account_if_needed(
                payer,
                system_program,
                desired_account_size,
            )?;

            Ok(event)
        }
    }

    pub fn new(
        receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
        user_receipt_token_account: &'a InterfaceAccount<'info, TokenAccount>,
        reward_account: &'a mut AccountLoader<'info, RewardAccount>,
        user_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,
    ) -> Result<Self> {
        Ok(Self {
            receipt_token_mint,
            user_receipt_token_account,
            reward_account,
            user_reward_account,
            _current_slot: Clock::get()?.slot,
        })
    }

    pub fn process_initialize_user_reward_account(
        &mut self,
        user_reward_account_bump: u8,
    ) -> Result<Option<events::UserCreatedOrUpdatedRewardAccount>> {
        if self.user_reward_account.as_ref().data_len()
            < 8 + std::mem::size_of::<UserRewardAccount>()
        {
            self.user_reward_account
                .initialize_zero_copy_header(user_reward_account_bump)?;
        } else {
            // initialize account
            self.user_reward_account
                .load_init()?
                .initialize(user_reward_account_bump, self.user_receipt_token_account)?;
            self.user_reward_account.exit(&crate::ID)?;

            // reflect existing token amount
            RewardService::new(self.receipt_token_mint, self.reward_account)?
                .update_reward_pools_token_allocation(
                    None,
                    Some(self.user_reward_account),
                    self.user_receipt_token_account.amount,
                    None,
                )?;

            return Ok(Some(events::UserCreatedOrUpdatedRewardAccount {
                receipt_token_mint: self.receipt_token_mint.key(),
                user_reward_account: self.user_reward_account.key(),
                receipt_token_amount: self.user_receipt_token_account.amount,
                created: true,
            }));
        }

        Ok(None)
    }

    pub fn process_update_user_reward_account_if_needed(
        &mut self,
        payer: &Signer<'info>,
        system_program: &Program<'info, System>,
        desired_account_size: Option<u32>,
    ) -> Result<Option<events::UserCreatedOrUpdatedRewardAccount>> {
        let min_account_size = 8 + std::mem::size_of::<UserRewardAccount>();
        let target_account_size = desired_account_size
            .map(|size| std::cmp::max(size as usize, min_account_size))
            .unwrap_or(min_account_size);

        let new_account_size = system_program.expand_account_size_if_needed(
            self.user_reward_account.as_ref(),
            payer,
            &[],
            target_account_size,
            None,
        )?;

        if new_account_size >= min_account_size {
            let (initializing, updated) = {
                let mut user_reward_account = self.user_reward_account.load_mut()?;
                let initializing = user_reward_account.is_initializing();

                if !initializing {
                    // Constraint check
                    // has_one = receipt_token_mint
                    // has_one = user
                    require_keys_eq!(
                        user_reward_account.receipt_token_mint,
                        self.receipt_token_mint.key(),
                    );
                    require_keys_eq!(
                        user_reward_account.user,
                        self.user_receipt_token_account.owner
                    );
                }

                let updated =
                    user_reward_account.update_if_needed(self.user_receipt_token_account)?;
                (initializing, updated)
            };

            // reflect existing token amount
            if initializing {
                RewardService::new(self.receipt_token_mint, self.reward_account)?
                    .update_reward_pools_token_allocation(
                        None,
                        Some(self.user_reward_account),
                        self.user_receipt_token_account.amount,
                        None,
                    )?;
            }

            if updated {
                return Ok(Some(events::UserCreatedOrUpdatedRewardAccount {
                    receipt_token_mint: self.receipt_token_mint.key(),
                    user_reward_account: self.user_reward_account.key(),
                    receipt_token_amount: self.user_receipt_token_account.amount,
                    created: initializing,
                }));
            }
        }

        Ok(None)
    }
}
