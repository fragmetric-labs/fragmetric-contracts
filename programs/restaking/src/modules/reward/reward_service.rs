use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::events;

use super::*;

pub struct RewardService<'info: 'a, 'a> {
    receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
    reward_account: &'a mut AccountLoader<'info, RewardAccount>,

    current_slot: u64,
}

impl<'info, 'a> RewardService<'info, 'a> {
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

    pub fn process_update_reward_pools(&self) -> Result<()> {
        self.reward_account
            .load_mut()?
            .update_reward_pools(self.current_slot)?;

        emit!(events::OperatorUpdatedRewardPools {
            receipt_token_mint: self.receipt_token_mint.key(),
            reward_account_address: self.reward_account.key(),
        });

        Ok(())
    }

    pub(in crate::modules) fn update_reward_pools_token_allocation(
        &self,
        from_user_reward_account: Option<&mut AccountLoader<UserRewardAccount>>,
        to_user_reward_account: Option<&mut AccountLoader<UserRewardAccount>>,
        amount: u64,
        contribution_accrual_rate: Option<u8>,
    ) -> Result<()> {
        let mut updated_user_reward_account_addresses = vec![];
        if let Some(from) = from_user_reward_account.as_ref() {
            updated_user_reward_account_addresses.push(from.key());
        }
        if let Some(to) = to_user_reward_account.as_ref() {
            updated_user_reward_account_addresses.push(to.key());
        }

        let mut from_account_ref = from_user_reward_account
            .map(|loader| loader.load_mut())
            .transpose()?;
        let from_account = from_account_ref.as_deref_mut();

        let mut to_account_ref = to_user_reward_account
            .map(|loader| loader.load_mut())
            .transpose()?;
        let to_account = to_account_ref.as_deref_mut();

        self.reward_account
            .load_mut()?
            .update_reward_pools_token_allocation(
                self.receipt_token_mint.key(),
                amount,
                contribution_accrual_rate,
                from_account,
                to_account,
                self.current_slot,
            )?;

        emit!(events::UserUpdatedRewardPool {
            receipt_token_mint: self.receipt_token_mint.key(),
            updated_user_reward_account_addresses,
        });

        Ok(())
    }
}
