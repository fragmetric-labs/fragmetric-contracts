use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::errors::ErrorCode;
use crate::events;

use super::*;

pub struct UserRewardService<'a, 'info> {
    receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
    reward_account: &'a AccountLoader<'info, RewardAccount>,
    user_reward_account: &'a AccountLoader<'info, UserRewardAccount>,
    current_slot: u64,
}

impl<'a, 'info> UserRewardService<'a, 'info> {
    pub fn new(
        receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
        user: &AccountInfo,
        reward_account: &'a AccountLoader<'info, RewardAccount>,
        user_reward_account: &'a AccountLoader<'info, UserRewardAccount>,
    ) -> Result<Self> {
        Self::validate_user_reward_account(
            receipt_token_mint,
            user,
            reward_account,
            user_reward_account,
        )?;

        let clock = Clock::get()?;
        Ok(Self {
            receipt_token_mint,
            reward_account,
            user_reward_account,
            current_slot: clock.slot,
        })
    }

    /// Validate the provided accounts have proper relationships.
    pub fn validate_user_reward_account(
        receipt_token_mint: &InterfaceAccount<Mint>,
        user: &AccountInfo,
        reward_account: &AccountLoader<RewardAccount>,
        user_reward_account: &AccountLoader<UserRewardAccount>,
    ) -> Result<()> {
        RewardService::validate_reward_account(receipt_token_mint, reward_account)?;

        let user_reward_account = user_reward_account.load()?;

        // has_one = receipt_token_mint
        // has_one = user
        // constraint = user_reward_account.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError
        require_keys_eq!(
            user_reward_account.receipt_token_mint,
            receipt_token_mint.key()
        );
        require_keys_eq!(user_reward_account.user, user.key());
        require!(
            user_reward_account.is_latest_version(),
            ErrorCode::InvalidAccountDataVersionError,
        );

        Ok(())
    }

    pub fn process_update_user_reward_pools(
        &self,
        mut num_blocks_to_settle: Option<u16>,
    ) -> Result<events::UserUpdatedRewardPool> {
        self.user_reward_account
            .load_mut()?
            .update_user_reward_pools(
                &mut *self.reward_account.load_mut()?,
                self.current_slot,
                &mut num_blocks_to_settle,
            )?;

        Ok(events::UserUpdatedRewardPool {
            receipt_token_mint: self.receipt_token_mint.key(),
            updated_user_reward_accounts: vec![self.user_reward_account.key()],
        })
    }

    pub fn process_claim_user_reward(
        &self,
        reward_token_mint: &InterfaceAccount<'info, Mint>,
        reward_token_program: &Interface<'info, TokenInterface>,
        reward_reserve_account: &SystemAccount<'info>,
        reward_token_reserve_account: &InterfaceAccount<'info, TokenAccount>,
        destination_reward_token_account: &InterfaceAccount<'info, TokenAccount>,
        authority: &Signer,
        is_bonus_pool: bool,
        amount: Option<u64>,
    ) -> Result<events::UserClaimedReward> {
        let mut reward_account = self.reward_account.load_mut()?;
        let mut user_reward_account = self.user_reward_account.load_mut()?;
        let reward_id = reward_account.get_reward_id(&reward_token_mint.key())?;

        let (claimed_amount, total_claimed_amount) = user_reward_account.claim_reward(
            &mut reward_account,
            authority.key,
            reward_id,
            is_bonus_pool,
            amount,
            self.current_slot,
        )?;

        anchor_spl::token_interface::transfer_checked(
            CpiContext::new_with_signer(
                reward_token_program.to_account_info(),
                anchor_spl::token_interface::TransferChecked {
                    from: reward_token_reserve_account.to_account_info(),
                    mint: reward_token_mint.to_account_info(),
                    to: destination_reward_token_account.to_account_info(),
                    authority: reward_reserve_account.to_account_info(),
                },
                &[&reward_account.get_reserve_account_seeds()],
            ),
            claimed_amount,
            reward_token_mint.decimals,
        )?;

        Ok(events::UserClaimedReward {
            receipt_token_mint: self.receipt_token_mint.key(),
            user: user_reward_account.user,
            reward_token_mint: reward_token_mint.key(),
            destination_reward_token_account: destination_reward_token_account.key(),
            destination_reward_token_account_owner: destination_reward_token_account.owner,
            updated_reward_account: self.reward_account.key(),
            updated_user_reward_account: self.user_reward_account.key(),
            claimed_reward_token_amount: claimed_amount,
            total_claimed_reward_token_amount: total_claimed_amount,
        })
    }
}
