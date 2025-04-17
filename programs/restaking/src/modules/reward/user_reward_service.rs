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

impl Drop for UserRewardService<'_, '_> {
    fn drop(&mut self) {
        self.reward_account.exit(&crate::ID).unwrap();
        self.user_reward_account.exit(&crate::ID).unwrap();
    }
}

impl<'a, 'info> UserRewardService<'a, 'info> {
    pub fn new(
        receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
        user: &UncheckedAccount<'info>,
        reward_account: &'a AccountLoader<'info, RewardAccount>,
        user_reward_account: &'a AccountLoader<'info, UserRewardAccount>,
    ) -> Result<Self> {
        Self::validate_user_reward_account(
            receipt_token_mint,
            user.key,
            reward_account,
            user_reward_account,
        )?;
        Self::validate_signer(&user, None)?;

        let clock = Clock::get()?;
        Ok(Self {
            receipt_token_mint,
            reward_account,
            user_reward_account,
            current_slot: clock.slot,
        })
    }

    /// Validate the provided accounts have proper relationships.
    pub(crate) fn validate_user_reward_account(
        receipt_token_mint: &InterfaceAccount<Mint>,
        user: &Pubkey,
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
        require_keys_eq!(user_reward_account.user, *user);
        require!(
            user_reward_account.is_latest_version(),
            ErrorCode::InvalidAccountDataVersionError,
        );

        Ok(())
    }

    pub(crate) fn validate_delegate(
        current_authority: &AccountInfo,
        authority_signer_seeds: Option<&[&[u8]]>,
        user_reward_account: &AccountLoader<UserRewardAccount>,
    ) -> Result<()> {
        Self::validate_signer(current_authority, authority_signer_seeds)?;

        // If delegate is set at user_reward_account, then check if delegate key is same as stored value.
        let user_reward_account = user_reward_account.load()?;
        if user_reward_account.is_delegate_set() {
            user_reward_account.validate_authority(current_authority.key())?;
        }

        Ok(())
    }

    fn validate_signer(
        authority: &AccountInfo,
        authority_signer_seeds: Option<&[&[u8]]>,
    ) -> Result<()> {
        if !authority.is_signer {
            let authority_signer_seeds =
                authority_signer_seeds.ok_or_else(|| ProgramError::MissingRequiredSignature)?;
            require_keys_eq!(
                authority.key(),
                Pubkey::create_program_address(authority_signer_seeds, &crate::ID)
                    .map_err(|_| ProgramError::InvalidSeeds)?
            );
        }

        Ok(())
    }

    pub fn process_update_user_reward_pools(&self) -> Result<events::UserUpdatedRewardPool> {
        self.user_reward_account
            .load_mut()?
            .update_user_reward_pools(&mut *self.reward_account.load_mut()?, self.current_slot)?;

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
        claim_authority: &Signer<'info>,
        is_bonus_pool: bool,
        amount: Option<u64>,
    ) -> Result<events::UserClaimedReward> {
        Self::validate_delegate(claim_authority, None, self.user_reward_account)?;

        let reward_token_mint_key = reward_token_mint.key();
        let mut reward_account = self.reward_account.load_mut()?;
        let mut user_reward_account = self.user_reward_account.load_mut()?;

        let reward = reward_account.get_reward_by_mint(&reward_token_mint_key)?;
        let reward_id = reward.id;

        require_keys_eq!(
            reward_token_reserve_account.key(),
            reward_account.find_reward_token_reserve_account_address(&reward_token_mint_key)?,
        );

        require_eq!(reward.claimable, 1, ErrorCode::RewardNotClaimableError);

        user_reward_account.backfill_not_existing_pools(&reward_account)?;

        let reward_pool = reward_account.get_reward_pool_mut(is_bonus_pool)?;
        let user_reward_pool = user_reward_account.get_user_reward_pool_mut(is_bonus_pool)?;
        let (claimed_amount, total_claimed_amount) =
            user_reward_pool.claim_reward(reward_pool, reward_id, self.current_slot, amount)?;

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
            reward_token_mint: reward_token_mint.key(),
            updated_reward_account: self.reward_account.key(),
            updated_user_reward_account: self.user_reward_account.key(),
            claimed_reward_token_amount: claimed_amount,
            total_claimed_reward_token_amount: total_claimed_amount,
        })
    }
}
