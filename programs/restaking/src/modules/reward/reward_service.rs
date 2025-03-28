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
        // Contribution accrual rate is only allowed for deposits
        if contribution_accrual_rate.is_some()
            && !(from_user_reward_account.is_none() && to_user_reward_account.is_some())
        {
            err!(ErrorCode::RewardInvalidTransferArgsException)?
        }

        if amount == 0 {
            return Ok(vec![]);
        }

        let mut reward_account = self.reward_account.load_mut()?;
        let mut updated_user_reward_accounts = Vec::with_capacity(2);

        if let Some(from_user_reward_account) = &from_user_reward_account {
            let mut from = from_user_reward_account.load_mut()?;

            require_keys_eq!(self.receipt_token_mint.key(), from.receipt_token_mint);

            from.backfill_not_existing_pools(&reward_account)?;
            for reward_pool in reward_account.get_reward_pools_iter_mut() {
                let user_reward_pool = from.get_user_reward_pool_mut(reward_pool.id)?;

                let effective_deltas = user_reward_pool.update_token_allocated_amount(
                    reward_pool,
                    vec![TokenAllocatedAmountDelta::new_negative(amount)],
                    self.current_slot,
                )?;
                reward_pool.update_token_allocated_amount(effective_deltas, self.current_slot)?;
            }

            updated_user_reward_accounts.push(from_user_reward_account.key());
        }

        if let Some(to_user_reward_account) = &to_user_reward_account {
            let mut to = to_user_reward_account.load_mut()?;

            require_keys_eq!(self.receipt_token_mint.key(), to.receipt_token_mint);

            to.backfill_not_existing_pools(&reward_account)?;
            for reward_pool in reward_account.get_reward_pools_iter_mut() {
                let user_reward_pool = to.get_user_reward_pool_mut(reward_pool.id)?;
                let effective_contribution_accrual_rate =
                    (reward_pool.custom_contribution_accrual_rate_enabled == 1)
                        .then_some(contribution_accrual_rate)
                        .flatten();

                let effective_deltas = user_reward_pool.update_token_allocated_amount(
                    reward_pool,
                    vec![TokenAllocatedAmountDelta::new_positive(
                        effective_contribution_accrual_rate,
                        amount,
                    )],
                    self.current_slot,
                )?;
                reward_pool.update_token_allocated_amount(effective_deltas, self.current_slot)?;
            }

            updated_user_reward_accounts.push(to_user_reward_account.key());
        }

        Ok(updated_user_reward_accounts)
    }
}
