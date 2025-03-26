use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::events;
use crate::utils::{AccountLoaderExt, SystemProgramExt};

use super::*;

pub struct RewardConfigurationService<'a, 'info> {
    receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
    reward_account: &'a mut AccountLoader<'info, RewardAccount>,

    current_slot: u64,
}

impl Drop for RewardConfigurationService<'_, '_> {
    fn drop(&mut self) {
        self.reward_account.exit(&crate::ID).unwrap();
    }
}

impl<'a, 'info> RewardConfigurationService<'a, 'info> {
    pub fn new(
        receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
        reward_account: &'a mut AccountLoader<'info, RewardAccount>,
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
            self.reward_account
                .load_init()?
                .initialize(reward_account_bump, self.receipt_token_mint.key())?;
            self.reward_account.exit(&crate::ID)
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
                .update_if_needed(self.receipt_token_mint.key())?;
        }

        Ok(())
    }

    pub fn process_add_reward_pool(
        &self,
        name: String,
        custom_contribution_accrual_rate_enabled: bool,
    ) -> Result<events::FundManagerUpdatedRewardPool> {
        self.reward_account.load_mut()?.add_reward_pool(
            name,
            custom_contribution_accrual_rate_enabled,
            self.current_slot,
        )?;

        self.create_fund_manager_updated_reward_pool_event()
    }

    pub fn process_add_reward(
        &self,
        name: String,
        description: String,
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
    ) -> Result<events::FundManagerUpdatedRewardPool> {
        self.reward_account
            .load_mut()?
            .add_reward(name, description, mint, program, decimals)?;

        self.create_fund_manager_updated_reward_pool_event()
    }

    pub fn process_settle_reward(
        &self,
        reward_pool_id: u8,
        reward_id: u16,
        amount: u64,
    ) -> Result<events::FundManagerUpdatedRewardPool> {
        // TODO v0.5/reward: ensure substantial asset transfer for certain type of rewards
        self.reward_account
            .load_mut()?
            .get_reward_pool_mut(reward_pool_id)?
            .settle_reward(reward_id, amount, self.current_slot)?;

        self.create_fund_manager_updated_reward_pool_event()
    }

    fn create_fund_manager_updated_reward_pool_event(
        &self,
    ) -> Result<events::FundManagerUpdatedRewardPool> {
        Ok(events::FundManagerUpdatedRewardPool {
            receipt_token_mint: self.receipt_token_mint.key(),
            reward_account: self.reward_account.key(),
        })
    }
}
