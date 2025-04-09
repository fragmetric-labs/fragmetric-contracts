use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::errors;
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
            let mut reward_account = self.reward_account.load_mut()?;
            reward_account.update_if_needed(self.receipt_token_mint.key())?;

            reward_account.set_reward_pool_idempotent(false, self.current_slot)?;
            reward_account.set_reward_pool_idempotent(true, self.current_slot)?;
        }

        Ok(())
    }

    pub fn process_add_reward(
        &self,
        reward_token_mint: Option<&InterfaceAccount<'info, Mint>>,
        reward_token_program: Option<&Interface<'info, TokenInterface>>,
        reward_token_reserve_account: Option<&InterfaceAccount<'info, TokenAccount>>,

        name: String,
        description: String,
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        claimable: bool,
    ) -> Result<events::FundManagerUpdatedRewardPool> {
        if claimable {
            // Constraint check
            let reward_token_mint =
                reward_token_mint.ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;
            let reward_token_program =
                reward_token_program.ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;
            let reward_token_reserve_account = reward_token_reserve_account
                .ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;

            require_keys_eq!(reward_token_reserve_account.mint, mint);
            require_keys_eq!(reward_token_program.key(), program);
            require_eq!(reward_token_mint.decimals, decimals);
        }

        self.reward_account.load_mut()?.add_reward(
            name,
            description,
            mint,
            program,
            decimals,
            claimable,
        )?;

        self.create_fund_manager_updated_reward_pool_event()
    }

    pub fn process_update_reward(
        &self,
        reward_token_mint: Option<&InterfaceAccount<'info, Mint>>,
        reward_token_program: Option<&Interface<'info, TokenInterface>>,
        reward_token_reserve_account: Option<&InterfaceAccount<'info, TokenAccount>>,

        reward_id: u16,
        mint: Option<Pubkey>,
        program: Option<Pubkey>,
        decimals: Option<u8>,
        claimable: bool,
    ) -> Result<events::FundManagerUpdatedRewardPool> {
        let mut reward_account = self.reward_account.load_mut()?;
        let total_unclaimed_amount = reward_account.get_total_reward_unclaimed_amount(reward_id);
        let reward = reward_account.get_reward_mut(reward_id)?;

        require_eq!(
            reward.claimable,
            0,
            errors::ErrorCode::RewardAlreadyClaimableError
        );

        let mint = mint.unwrap_or(reward.mint);
        let program = program.unwrap_or(reward.program);
        let decimals = decimals.unwrap_or(reward.decimals);

        reward
            .set_claimable(claimable)
            .set_reward_token(mint, program, decimals);

        drop(reward_account);

        if claimable {
            let reward_token_mint =
                reward_token_mint.ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;
            let reward_token_program =
                reward_token_program.ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;
            let reward_token_reserve_account = reward_token_reserve_account
                .ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;

            // Constraint check
            require_keys_eq!(reward_token_reserve_account.mint, mint);
            require_keys_eq!(reward_token_program.key(), program);
            require_eq!(reward_token_mint.decimals, decimals);

            // assert unclaimed amount <= ATA balance
            require_gte!(
                reward_token_reserve_account.amount,
                total_unclaimed_amount,
                errors::ErrorCode::RewardNotEnoughRewardsToClaimError,
            );
        }

        self.create_fund_manager_updated_reward_pool_event()
    }

    pub fn process_settle_reward(
        &mut self,
        reward_token_mint: Option<&InterfaceAccount<'info, Mint>>,
        reward_token_program: Option<&Interface<'info, TokenInterface>>,
        reward_token_reserve_account: Option<&InterfaceAccount<'info, TokenAccount>>,

        reward_pool_id: u8,
        reward_id: u16,
        amount: u64,
    ) -> Result<events::FundManagerUpdatedRewardPool> {
        let (total_unclaimed_amount, mint, program, decimals, claimable) = {
            let reward_account = self.reward_account.load_mut()?;
            let total_unclaimed_amount =
                reward_account.get_total_reward_unclaimed_amount(reward_id);
            let reward = reward_account.get_reward(reward_id)?;
            (
                total_unclaimed_amount,
                reward.mint,
                reward.program,
                reward.decimals,
                reward.claimable,
            )
        };

        if claimable == 1 {
            let reward_token_mint =
                reward_token_mint.ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;
            let reward_token_program =
                reward_token_program.ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;
            let reward_token_reserve_account = reward_token_reserve_account
                .ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;

            // Constraint check
            require_keys_eq!(reward_token_reserve_account.mint, mint);
            require_keys_eq!(reward_token_program.key(), program);
            require_eq!(reward_token_mint.decimals, decimals);

            require_gte!(
                reward_token_reserve_account.amount,
                total_unclaimed_amount + amount,
                errors::ErrorCode::RewardNotEnoughRewardsToClaimError,
            );
        }

        self.settle_reward(reward_pool_id, reward_id, amount)?;

        self.create_fund_manager_updated_reward_pool_event()
    }

    /// Settle reward.
    pub(in crate::modules) fn settle_reward(
        &self,
        reward_pool_id: u8,
        reward_id: u16,
        amount: u64,
    ) -> Result<()> {
        self.reward_account
            .load_mut()?
            .get_reward_pool_mut(reward_pool_id)?
            .settle_reward(reward_id, amount, self.current_slot)
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
