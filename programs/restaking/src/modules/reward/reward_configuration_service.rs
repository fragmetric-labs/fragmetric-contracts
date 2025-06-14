use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::events;
use crate::utils::{AccountLoaderExt, SystemProgramExt};

use super::*;

pub struct RewardConfigurationService<'a, 'info> {
    receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
    reward_account: &'a AccountLoader<'info, RewardAccount>,

    current_slot: u64,
}

impl<'a, 'info> RewardConfigurationService<'a, 'info> {
    pub fn new(
        receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
        reward_account: &'a AccountLoader<'info, RewardAccount>,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            receipt_token_mint,
            reward_account,
            current_slot: clock.slot,
        })
    }

    pub fn process_initialize_reward_account(&mut self, reward_account_bump: u8) -> Result<()> {
        if self.reward_account.as_ref().data_len() < 8 + std::mem::size_of::<RewardAccount>() {
            self.reward_account
                .initialize_zero_copy_header(reward_account_bump)
        } else {
            self.reward_account.load_init()?.initialize(
                reward_account_bump,
                self.receipt_token_mint.key(),
                self.current_slot,
            )
        }
    }

    pub fn process_update_reward_account_if_needed(
        &self,
        payer: &Signer<'info>,
        system_program: &Program<'info, System>,
        desired_account_size: Option<u32>,
    ) -> Result<()> {
        let min_account_size = 8 + std::mem::size_of::<RewardAccount>();
        let target_account_size = desired_account_size
            .map(|size| std::cmp::max(size as usize, min_account_size))
            .unwrap_or(min_account_size);

        let new_account_size = system_program.expand_account_size_if_needed(
            self.reward_account.as_ref(),
            payer,
            &[],
            target_account_size,
            None,
        )?;

        if new_account_size >= min_account_size {
            self.reward_account
                .load_mut()?
                .update_if_needed(self.receipt_token_mint.key(), self.current_slot)?;
        }

        Ok(())
    }

    pub fn process_add_reward(
        &self,
        reward_token_mint: Option<&InterfaceAccount<Mint>>,
        reward_token_program: Option<&Interface<TokenInterface>>,
        reward_token_reserve_account: Option<&InterfaceAccount<TokenAccount>>,

        name: String,
        description: String,
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        claimable: bool,
    ) -> Result<events::FundManagerUpdatedRewardPool> {
        let reward_id = self.reward_account.load_mut()?.add_reward(
            name,
            description,
            mint,
            program,
            decimals,
            claimable,
        )?;

        if claimable {
            RewardService::new(self.receipt_token_mint, self.reward_account)?
                .validate_reward_token_reserve_account(
                    reward_token_mint,
                    reward_token_program,
                    reward_token_reserve_account,
                    reward_id,
                )?;
        }

        Ok(events::FundManagerUpdatedRewardPool {
            receipt_token_mint: self.receipt_token_mint.key(),
            reward_account: self.reward_account.key(),
        })
    }

    pub fn process_update_reward(
        &self,
        reward_token_mint: Option<&InterfaceAccount<Mint>>,
        reward_token_program: Option<&Interface<TokenInterface>>,
        reward_token_reserve_account: Option<&InterfaceAccount<TokenAccount>>,

        mint: Pubkey,
        new_mint: Option<Pubkey>,
        new_program: Option<Pubkey>,
        new_decimals: Option<u8>,
        claimable: bool,
    ) -> Result<events::FundManagerUpdatedRewardPool> {
        let mut reward_account = self.reward_account.load_mut()?;
        let reward_id = reward_account.get_reward_id(&mint)?;

        reward_account.update_reward(reward_id, new_mint, new_program, new_decimals, claimable)?;

        drop(reward_account);

        if claimable {
            RewardService::new(self.receipt_token_mint, self.reward_account)?
                .validate_reward_token_reserve_account(
                    reward_token_mint,
                    reward_token_program,
                    reward_token_reserve_account,
                    reward_id,
                )?;
        }

        Ok(events::FundManagerUpdatedRewardPool {
            receipt_token_mint: self.receipt_token_mint.key(),
            reward_account: self.reward_account.key(),
        })
    }
}
