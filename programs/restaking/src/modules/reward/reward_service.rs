use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::errors::ErrorCode;
use crate::events;

use super::*;

pub struct RewardService<'a, 'info> {
    receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
    reward_account: &'a AccountLoader<'info, RewardAccount>,

    current_slot: u64,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct RewardSettlementBlockSlotAndContribution {
    pub starting_slot: u64,
    pub ending_slot: u64,
    pub starting_reward_pool_contribution: u128,
    pub ending_reward_pool_contribution: u128,
}

impl<'a, 'info> RewardService<'a, 'info> {
    pub fn new(
        receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
        reward_account: &'a AccountLoader<'info, RewardAccount>,
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
        from_user_reward_account: Option<&AccountLoader<UserRewardAccount>>,
        to_user_reward_account: Option<&AccountLoader<UserRewardAccount>>,
        amount: u64,
        contribution_accrual_rate: Option<u16>,
    ) -> Result<Vec<Pubkey>> {
        // Contribution accrual rate is only allowed for deposits
        if contribution_accrual_rate.is_some()
            && !(from_user_reward_account.is_none() && to_user_reward_account.is_some())
        {
            err!(ErrorCode::RewardInvalidTransferArgsException)?
        }

        if amount == 0 || from_user_reward_account.is_none() && to_user_reward_account.is_none() {
            return Ok(vec![]);
        }

        let mut reward_account = self.reward_account.load_mut()?;
        let mut updated_user_reward_accounts = Vec::with_capacity(2);

        if let Some(from_user_reward_account) = &from_user_reward_account {
            let mut from = from_user_reward_account.load_mut()?;

            require_keys_eq!(self.receipt_token_mint.key(), from.receipt_token_mint);

            let reward_pools_iter = reward_account.get_reward_pools_iter_mut();
            let from_user_reward_pools_iter = from.get_user_reward_pools_iter_mut();
            for (reward_pool, user_reward_pool) in
                reward_pools_iter.zip(from_user_reward_pools_iter)
            {
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

            let reward_pools_iter = reward_account.get_reward_pools_iter_mut();
            let to_user_reward_pools_iter = to.get_user_reward_pools_iter_mut();
            for (reward_pool, user_reward_pool) in reward_pools_iter.zip(to_user_reward_pools_iter)
            {
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

    pub fn process_settle_reward(
        &self,
        reward_token_mint: Option<&InterfaceAccount<Mint>>,
        reward_token_program: Option<&Interface<TokenInterface>>,
        reward_token_reserve_account: Option<&InterfaceAccount<TokenAccount>>,

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

        Ok(events::FundManagerUpdatedRewardPool {
            receipt_token_mint: self.receipt_token_mint.key(),
            reward_account: self.reward_account.key(),
        })
    }

    /// Settle reward.
    pub(in crate::modules) fn settle_reward(
        &self,
        reward_token_mint: Option<&InterfaceAccount<Mint>>,
        reward_token_program: Option<&Interface<TokenInterface>>,
        reward_token_reserve_account: Option<&InterfaceAccount<TokenAccount>>,

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

    pub fn get_last_settled_block_slot_and_contribution(
        &self,
        reward_token_mint: &Pubkey,
        is_bonus_pool: bool,
    ) -> Result<RewardSettlementBlockSlotAndContribution> {
        let reward_account = self.reward_account.load()?;
        let reward_id = reward_account.get_reward_id(reward_token_mint)?;
        let reward_pool = reward_account.get_reward_pool(is_bonus_pool);
        let reward_settlement = reward_pool.get_reward_settlement(reward_id)?;
        let last_settled_block = reward_settlement.get_tail_settlement_block();

        Ok(RewardSettlementBlockSlotAndContribution {
            starting_slot: last_settled_block.starting_slot,
            ending_slot: last_settled_block.ending_slot,
            starting_reward_pool_contribution: last_settled_block.starting_reward_pool_contribution,
            ending_reward_pool_contribution: last_settled_block.ending_reward_pool_contribution,
        })
    }

    pub fn process_claim_remaining_reward(
        &self,
        reward_token_mint: &InterfaceAccount<'info, Mint>,
        reward_token_program: &Interface<'info, TokenInterface>,
        reward_reserve_account: &SystemAccount<'info>,
        reward_token_reserve_account: &InterfaceAccount<'info, TokenAccount>,
        program_reward_token_revenue_account: &InterfaceAccount<'info, TokenAccount>,
    ) -> Result<()> {
        self.claim_remaining_reward(
            reward_token_mint,
            reward_token_program,
            reward_reserve_account,
            reward_token_reserve_account,
            program_reward_token_revenue_account,
        )
    }

    pub(in crate::modules) fn claim_remaining_reward(
        &self,
        reward_token_mint: &InterfaceAccount<'info, Mint>,
        reward_token_program: &Interface<'info, TokenInterface>,
        reward_reserve_account: &SystemAccount<'info>,
        reward_token_reserve_account: &InterfaceAccount<'info, TokenAccount>,
        program_reward_token_revenue_account: &InterfaceAccount<'info, TokenAccount>,
    ) -> Result<()> {
        let mut reward_account = self.reward_account.load_mut()?;
        let reward_id = reward_account.get_reward_id(&reward_token_mint.key())?;

        let claimed_amount = reward_account.claim_remaining_reward(reward_id, self.current_slot)?;

        anchor_spl::token_interface::transfer_checked(
            CpiContext::new_with_signer(
                reward_token_program.to_account_info(),
                anchor_spl::token_interface::TransferChecked {
                    from: reward_token_reserve_account.to_account_info(),
                    mint: reward_token_mint.to_account_info(),
                    to: program_reward_token_revenue_account.to_account_info(),
                    authority: reward_reserve_account.to_account_info(),
                },
                &[&reward_account.get_reserve_account_seeds()],
            ),
            claimed_amount,
            reward_token_mint.decimals,
        )?;

        drop(reward_account);

        self.validate_reward_token_reserve_account(
            Some(reward_token_mint),
            Some(reward_token_program),
            Some(reward_token_reserve_account),
            reward_id,
        )?;

        Ok(())
    }

    pub(super) fn validate_reward_token_reserve_account(
        &self,
        reward_token_mint: Option<&InterfaceAccount<Mint>>,
        reward_token_program: Option<&Interface<TokenInterface>>,
        reward_token_reserve_account: Option<&InterfaceAccount<TokenAccount>>,
        reward_id: u16,
    ) -> Result<()> {
        let reward_account = self.reward_account.load()?;
        let reward_reserve_account_address = reward_account.get_reserve_account_address()?;

        let reward_token_mint =
            reward_token_mint.ok_or_else(|| error!(error::ErrorCode::ConstraintAccountIsNone))?;
        let reward_token_program = reward_token_program
            .ok_or_else(|| error!(error::ErrorCode::ConstraintAccountIsNone))?;
        let reward_token_reserve_account = reward_token_reserve_account
            .ok_or_else(|| error!(error::ErrorCode::ConstraintAccountIsNone))?;

        // Constraint check
        // associated_token::mint = reward_token_mint
        // associated_token::authority = reward_reserve_account
        // associated_token::token_program = reward_token_program
        require_keys_eq!(
            reward_token_reserve_account.owner,
            reward_reserve_account_address,
            error::ErrorCode::ConstraintTokenOwner,
        );
        require_keys_eq!(
            reward_token_reserve_account.key(),
            anchor_spl::associated_token::get_associated_token_address_with_program_id(
                &reward_reserve_account_address,
                &reward_token_mint.key(),
                reward_token_program.key,
            ),
            error::ErrorCode::ConstraintAssociated,
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
            ErrorCode::RewardNotEnoughRewardsToClaimError,
        );

        Ok(())
    }
}
