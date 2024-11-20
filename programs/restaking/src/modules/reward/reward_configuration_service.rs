use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};

use crate::errors::ErrorCode;
use crate::events;
use crate::utils::AccountLoaderExt;

use super::*;

pub struct RewardConfigurationService<'info: 'a, 'a> {
    receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
    reward_account: &'a mut AccountLoader<'info, RewardAccount>,

    current_slot: u64,
}

impl<'info, 'a> RewardConfigurationService<'info, 'a> {
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
                .initialize_zero_copy_header(reward_account_bump)?;
        } else {
            self.reward_account
                .load_init()?
                .initialize(reward_account_bump, self.receipt_token_mint.key());
        }
        Ok(())
    }

    pub fn process_update_reward_account_if_needed(
        &self,
        payer: &Signer<'info>,
        system_program: &Program<'info, System>,
        desired_account_size: Option<u32>,
    ) -> Result<()> {
        self.reward_account.expand_account_size_if_needed(
            payer,
            system_program,
            desired_account_size,
        )?;

        if self.reward_account.as_ref().data_len() >= 8 + std::mem::size_of::<RewardAccount>() {
            self.reward_account
                .load_mut()?
                .update_if_needed(self.receipt_token_mint.key());
        }

        Ok(())
    }

    pub fn process_add_reward_pool_holder(
        &self,
        name: String,
        description: String,
        pubkeys: Vec<Pubkey>,
    ) -> Result<()> {
        self.reward_account
            .load_mut()?
            .add_new_holder(name, description, pubkeys)?;

        self.emit_fund_manager_updated_reward_pool_event()
    }

    fn emit_fund_manager_updated_reward_pool_event(&self) -> Result<()> {
        emit!(events::FundManagerUpdatedRewardPool {
            receipt_token_mint: self.receipt_token_mint.key(),
            reward_account_address: self.reward_account.key(),
        });

        Ok(())
    }

    pub fn process_add_reward_pool(
        &self,
        name: String,
        holder_id: Option<u8>,
        custom_contribution_accrual_rate_enabled: bool,
    ) -> Result<()> {
        self.reward_account.load_mut()?.add_new_reward_pool(
            name,
            holder_id,
            custom_contribution_accrual_rate_enabled,
            self.current_slot,
        )?;

        self.emit_fund_manager_updated_reward_pool_event()
    }

    pub fn process_close_reward_pool(&self, reward_pool_id: u8) -> Result<()> {
        self.reward_account
            .load_mut()?
            .close_reward_pool(reward_pool_id, self.current_slot)?;

        self.emit_fund_manager_updated_reward_pool_event()
    }

    pub fn process_add_reward(
        &self,
        reward_token_mint: Option<&InterfaceAccount<Mint>>,
        reward_token_program: Option<&Interface<TokenInterface>>,
        name: String,
        description: String,
        reward_type: RewardType,
    ) -> Result<()> {
        Self::validate_token_reward_type(reward_token_mint, reward_token_program, &reward_type)?;

        self.reward_account
            .load_mut()?
            .add_new_reward(name, description, reward_type)?;

        self.emit_fund_manager_updated_reward_pool_event()
    }

    fn validate_token_reward_type(
        reward_token_mint: Option<&InterfaceAccount<Mint>>,
        reward_token_program: Option<&Interface<TokenInterface>>,
        reward_type: &RewardType,
    ) -> Result<()> {
        if let RewardType::Token {
            mint,
            program,
            decimals,
        } = reward_type
        {
            let (expected_mint, expected_program) = match (reward_token_mint, reward_token_program)
            {
                (Some(mint), Some(program)) => (mint, program),
                _ => err!(ErrorCode::RewardInvalidRewardTypeError)?,
            };

            require_keys_eq!(
                *mint,
                expected_mint.key(),
                ErrorCode::RewardInvalidRewardTypeError,
            );
            require_keys_eq!(
                *program,
                expected_program.key(),
                ErrorCode::RewardInvalidRewardTypeError,
            );
            require_eq!(
                *decimals,
                expected_mint.decimals,
                ErrorCode::RewardInvalidRewardTypeError,
            );
        }

        Ok(())
    }

    pub fn process_settle_reward(
        &self,
        reward_pool_id: u8,
        reward_id: u16,
        amount: u64,
    ) -> Result<()> {
        // TODO v0.4/reward: ensure substantial asset transfer for certain type of rewards

        self.reward_account.load_mut()?.settle_reward(
            reward_pool_id,
            reward_id,
            amount,
            self.current_slot,
        )?;

        self.emit_fund_manager_updated_reward_pool_event()
    }
}
