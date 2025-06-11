use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::errors;
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
        let reward_id = self.reward_account.load_mut()?.add_reward(
            name,
            description,
            mint,
            program,
            decimals,
            claimable,
        )?;

        if claimable {
            self.validate_reward_token_reserve_account(
                reward_token_mint,
                reward_token_program,
                reward_token_reserve_account,
                reward_id,
            )?;
        }

        self.create_fund_manager_updated_reward_pool_event()
    }

    pub fn process_update_reward(
        &self,
        reward_token_mint: Option<&InterfaceAccount<'info, Mint>>,
        reward_token_program: Option<&Interface<'info, TokenInterface>>,
        reward_token_reserve_account: Option<&InterfaceAccount<'info, TokenAccount>>,

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
            self.validate_reward_token_reserve_account(
                reward_token_mint,
                reward_token_program,
                reward_token_reserve_account,
                reward_id,
            )?;
        }

        self.create_fund_manager_updated_reward_pool_event()
    }

    pub fn process_settle_reward(
        &self,
        reward_token_mint: Option<&InterfaceAccount<'info, Mint>>,
        reward_token_program: Option<&Interface<'info, TokenInterface>>,
        reward_token_reserve_account: Option<&InterfaceAccount<'info, TokenAccount>>,

        mint: Pubkey,
        is_bonus_pool: bool,
        amount: u64,
    ) -> Result<events::FundManagerUpdatedRewardPool> {
        self.settle_reward(
            reward_token_mint,
            reward_token_program,
            reward_token_reserve_account,
            mint,
            is_bonus_pool,
            amount,
        )?;

        self.create_fund_manager_updated_reward_pool_event()
    }

    /// Settle reward.
    pub(in crate::modules) fn settle_reward(
        &self,
        reward_token_mint: Option<&InterfaceAccount<'info, Mint>>,
        reward_token_program: Option<&Interface<'info, TokenInterface>>,
        reward_token_reserve_account: Option<&InterfaceAccount<'info, TokenAccount>>,

        mint: Pubkey,
        is_bonus_pool: bool,
        amount: u64,
    ) -> Result<()> {
        let mut reward_account = self.reward_account.load_mut()?;
        let reward_id = reward_account.get_reward_id(&mint)?;
        let claimable = reward_account.get_reward(reward_id)?.claimable;

        reward_account.settle_reward(reward_id, is_bonus_pool, amount, self.current_slot)?;

        drop(reward_account);

        if claimable == 1 {
            self.validate_reward_token_reserve_account(
                reward_token_mint,
                reward_token_program,
                reward_token_reserve_account,
                reward_id,
            )?;
        }

        Ok(())
    }

    fn validate_reward_token_reserve_account(
        &self,
        reward_token_mint: Option<&InterfaceAccount<Mint>>,
        reward_token_program: Option<&Interface<TokenInterface>>,
        reward_token_reserve_account: Option<&InterfaceAccount<TokenAccount>>,
        reward_id: u16,
    ) -> Result<()> {
        let reward_account = self.reward_account.load()?;
        let reward_reserve_account_address = reward_account.get_reserve_account_address()?;

        let reward_token_mint =
            reward_token_mint.ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;
        let reward_token_program =
            reward_token_program.ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;
        let reward_token_reserve_account = reward_token_reserve_account
            .ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;

        // Constraint check
        // associated_token::mint = reward_token_mint
        // associated_token::authority = reward_reserve_account
        // associated_token::token_program = reward_token_program
        require_keys_eq!(
            reward_token_reserve_account.owner,
            reward_reserve_account_address,
            ErrorCode::ConstraintTokenOwner,
        );
        require_keys_eq!(
            reward_token_reserve_account.key(),
            anchor_spl::associated_token::get_associated_token_address_with_program_id(
                &reward_reserve_account_address,
                &reward_token_mint.key(),
                reward_token_program.key,
            ),
            ErrorCode::ConstraintAssociated,
        );

        // Check correct mint, program and decimals are provided
        let reward = reward_account.get_reward(reward_id)?;
        require_keys_eq!(reward_token_mint.key(), reward.mint);
        require_keys_eq!(reward_token_program.key(), reward.program);
        require_eq!(reward_token_mint.decimals, reward.decimals);

        // assert unclaimed amount <= ATA balance
        let unclaimed_reward_amount = reward_account.get_unclaimed_reward_amount(reward_id);
        require_gte!(
            reward_token_reserve_account.amount,
            unclaimed_reward_amount,
            errors::ErrorCode::RewardNotEnoughRewardsToClaimError,
        );

        Ok(())
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
