use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token::spl_token;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::utils::*;
use crate::{events, modules};

use super::*;

pub struct UserRewardConfigurationService<'a, 'info> {
    receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
    user_receipt_token_account: &'a InterfaceAccount<'info, TokenAccount>,
    reward_account: &'a AccountLoader<'info, RewardAccount>,
    user_reward_account: &'info AccountInfo<'info>,
}

impl<'a, 'info> UserRewardConfigurationService<'a, 'info> {
    pub fn new(
        receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
        user_receipt_token_account: &'a InterfaceAccount<'info, TokenAccount>,
        reward_account: &'a AccountLoader<'info, RewardAccount>,
        user_reward_account: &'info AccountInfo<'info>,
    ) -> Result<Self> {
        Ok(Self {
            receipt_token_mint,
            user_receipt_token_account,
            reward_account,
            user_reward_account,
        })
    }

    pub fn process_create_user_reward_account_idempotent(
        &self,
        system_program: &Program<'info, System>,
        payer: &Signer<'info>,
        user: &AccountInfo<'info>,
        user_reward_account_bump: u8,
        delegate: Option<Pubkey>,
        desired_account_size: Option<u32>,
    ) -> Result<Option<events::UserCreatedOrUpdatedRewardAccount>> {
        // validate delegation capability
        if delegate.is_some() {
            let user_owner = user.owner.key();

            // only the fund_wrap_account can be delegated on creation among system-owned accounts
            if user_owner == solana_program::system_program::id() {
                let (fund_wrap_account_address, _bump) = Pubkey::find_program_address(
                    &[
                        modules::fund::FundAccount::WRAP_SEED,
                        self.receipt_token_mint.key().as_ref(),
                    ],
                    &crate::id(),
                );
                require_keys_eq!(user.key(), fund_wrap_account_address);
            } else {
                // wrapped token accounts (SPL Token PDA) may be delegated for DeFi wrapped token tracking
                require_keys_eq!(user_owner, spl_token::id());
            }
        }

        let min_account_size = 8 + core::mem::size_of::<UserRewardAccount>();
        let new_account_size = if self.user_reward_account.is_initialized() {
            let user_reward_account =
                AccountLoader::<UserRewardAccount>::try_from(self.user_reward_account)?;

            // Constraint check
            // bump = user_reward_account.get_bump()?
            require_eq!(user_reward_account.get_bump()?, user_reward_account_bump);

            // expand account size if needed
            let target_account_size = desired_account_size
                .map(|size| min_account_size.max(size as usize))
                .unwrap_or(min_account_size);

            system_program.expand_account_size_if_needed(
                self.user_reward_account.as_ref(),
                payer,
                &[],
                target_account_size,
                None,
            )?
        } else {
            let new_account_size = core::cmp::min(
                solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE,
                8 + core::mem::size_of::<UserRewardAccount>(),
            );

            system_program.initialize_account(
                self.user_reward_account,
                payer,
                &[&[
                    UserRewardAccount::SEED,
                    self.receipt_token_mint.key().as_ref(),
                    self.user_receipt_token_account.owner.as_ref(),
                    &[user_reward_account_bump],
                ]],
                new_account_size,
                None,
                &crate::ID,
            )?;

            let user_reward_account = AccountLoader::<UserRewardAccount>::try_from_unchecked(
                &crate::ID,
                self.user_reward_account,
            )?;
            user_reward_account.initialize_zero_copy_header(user_reward_account_bump)?;
            user_reward_account.exit(&crate::ID)?;

            new_account_size
        };

        if new_account_size >= min_account_size {
            let user_reward_account =
                AccountLoader::<UserRewardAccount>::try_from(self.user_reward_account)?;

            let (initializing, updated) = {
                let mut user_reward_account = user_reward_account.load_mut()?;
                let initializing = user_reward_account.is_initializing();

                let updated = if initializing {
                    user_reward_account.initialize(
                        user_reward_account_bump,
                        &*self.reward_account.load()?,
                        self.user_receipt_token_account,
                        delegate,
                    )?
                } else {
                    // Constraint check
                    // has_one = receipt_token_mint
                    // has_one = user
                    require_keys_eq!(
                        user_reward_account.receipt_token_mint,
                        self.receipt_token_mint.key(),
                    );
                    require_keys_eq!(
                        user_reward_account.user,
                        self.user_receipt_token_account.owner,
                    );

                    user_reward_account.update_if_needed(
                        &*self.reward_account.load()?,
                        self.user_receipt_token_account,
                        delegate,
                    )?
                };

                (initializing, updated)
            };

            // reflect existing token amount
            if initializing {
                RewardService::new(self.receipt_token_mint, self.reward_account)?
                    .update_reward_pools_token_allocation(
                        None,
                        Some(&user_reward_account),
                        self.user_receipt_token_account.amount,
                        None,
                    )?;
            }

            if updated {
                return Ok(Some(events::UserCreatedOrUpdatedRewardAccount {
                    receipt_token_mint: self.receipt_token_mint.key(),
                    user: self.user_receipt_token_account.owner,
                    user_reward_account: user_reward_account.key(),
                    receipt_token_amount: self.user_receipt_token_account.amount,
                    created: initializing,
                }));
            }
        }

        Ok(None)
    }

    pub fn process_delegate_user_reward_account(
        &self,
        authority: &Signer,
        delegate: Option<Pubkey>,
    ) -> Result<events::UserDelegatedRewardAccount> {
        let user_reward_account =
            AccountLoader::<UserRewardAccount>::try_from(self.user_reward_account)?;

        user_reward_account
            .load_mut()?
            .set_delegate(authority.key, delegate)?;

        Ok(events::UserDelegatedRewardAccount {
            receipt_token_mint: self.receipt_token_mint.key(),
            user: self.user_receipt_token_account.owner,
            user_reward_account: self.user_reward_account.key(),
            delegate,
        })
    }
}
