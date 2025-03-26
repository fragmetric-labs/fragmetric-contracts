use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::errors::ErrorCode;
use crate::events;

use super::*;

pub struct RewardService<'a, 'info> {
    receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
    reward_account: &'a mut AccountLoader<'info, RewardAccount>,

    current_slot: u64,
}

impl Drop for RewardService<'_, '_> {
    fn drop(&mut self) {
        self.reward_account.exit(&crate::ID).unwrap();
    }
}

impl<'a, 'info> RewardService<'a, 'info> {
    pub fn new(
        receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
        reward_account: &'a mut AccountLoader<'info, RewardAccount>,
    ) -> Result<Self> {
        Self::validate_reward_account(receipt_token_mint, reward_account)?;

        let clock = Clock::get()?;
        Ok(Self {
            receipt_token_mint,
            reward_account,
            current_slot: clock.slot,
        })
    }

    pub fn validate_reward_account(
        receipt_token_mint: &InterfaceAccount<Mint>,
        reward_account: &AccountLoader<RewardAccount>,
    ) -> Result<()> {
        let reward_account = reward_account.load()?;

        // has_one = receipt_token_mint
        // constraint = reward_account.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError
        require_keys_eq!(reward_account.receipt_token_mint, receipt_token_mint.key());
        require!(
            reward_account.is_latest_version(),
            ErrorCode::InvalidAccountDataVersionError,
        );

        Ok(())
    }

    pub fn process_update_reward_pools(&self) -> Result<events::OperatorUpdatedRewardPools> {
        self.reward_account
            .load_mut()?
            .update_reward_pools(self.current_slot);

        Ok(events::OperatorUpdatedRewardPools {
            receipt_token_mint: self.receipt_token_mint.key(),
            reward_account: self.reward_account.key(),
        })
    }

    /// returns updated user reward accounts
    pub(in crate::modules) fn update_reward_pools_token_allocation(
        &self,
        from_user_reward_account: Option<&mut AccountLoader<UserRewardAccount>>,
        to_user_reward_account: Option<&mut AccountLoader<UserRewardAccount>>,
        amount: u64,
        contribution_accrual_rate: Option<u16>,
    ) -> Result<Vec<Pubkey>> {
        if amount == 0 {
            return Ok(vec![]);
        }

        let mut updated_user_reward_accounts = Vec::with_capacity(2);
        if let Some(from) = &from_user_reward_account {
            require_keys_eq!(
                self.receipt_token_mint.key(),
                from.load()?.receipt_token_mint,
            );
            updated_user_reward_accounts.push(from.key());
        }
        if let Some(to) = &to_user_reward_account {
            require_keys_eq!(self.receipt_token_mint.key(), to.load()?.receipt_token_mint);
            updated_user_reward_accounts.push(to.key());
        }

        let mut from_account_ref = from_user_reward_account
            .map(|loader| loader.load_mut())
            .transpose()?;
        let mut to_account_ref = to_user_reward_account
            .map(|loader| loader.load_mut())
            .transpose()?;

        self.reward_account
            .load_mut()?
            .update_reward_pools_token_allocation(
                amount,
                contribution_accrual_rate,
                from_account_ref.as_deref_mut(),
                to_account_ref.as_deref_mut(),
                self.current_slot,
            )?;

        Ok(updated_user_reward_accounts)
    }
}
