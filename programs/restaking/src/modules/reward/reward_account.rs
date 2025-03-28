use anchor_lang::prelude::*;
use anchor_spl::associated_token::spl_associated_token_account;

use crate::errors::ErrorCode;
use crate::utils::{PDASeeds, ZeroCopyHeader};

use super::*;

#[constant]
/// ## Version History
/// * v34: Initial Version (Data Size = 342072 ~= 335KB)
/// * v35: remove holder (Data Size = 348160 = 340KB)
pub const REWARD_ACCOUNT_CURRENT_VERSION: u16 = 35;
const REWARD_ACCOUNT_REWARDS_MAX_LEN_1: usize = 16;
const REWARD_ACCOUNT_REWARD_POOLS_MAX_LEN_1: usize = 4;

#[account(zero_copy)]
#[repr(C)]
pub struct RewardAccount {
    data_version: u16,
    bump: u8,
    pub receipt_token_mint: Pubkey,
    reserve_account_bump: u8,

    max_rewards: u16,
    max_reward_pools: u8,
    _padding1: u8,

    num_rewards: u16,
    num_reward_pools: u8,
    _padding2: [u8; 5],

    // informative
    reserve_account: Pubkey,

    _reserved: [u8; 2592],

    rewards_1: [Reward; REWARD_ACCOUNT_REWARDS_MAX_LEN_1],
    reward_pools_1: [RewardPool; REWARD_ACCOUNT_REWARD_POOLS_MAX_LEN_1],

    _reserved1: [u8; 6088],
}

impl PDASeeds<3> for RewardAccount {
    const SEED: &'static [u8] = b"reward";

    fn get_bump(&self) -> u8 {
        self.bump
    }

    fn get_seeds(&self) -> [&[u8]; 3] {
        [
            Self::SEED,
            self.receipt_token_mint.as_ref(),
            std::slice::from_ref(&self.bump),
        ]
    }
}

impl ZeroCopyHeader for RewardAccount {
    fn get_bump_offset() -> usize {
        2
    }
}

impl RewardAccount {
    fn migrate(&mut self, bump: u8, receipt_token_mint: Pubkey) -> Result<()> {
        if self.data_version == 0 {
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
            self.max_rewards = REWARD_ACCOUNT_REWARDS_MAX_LEN_1 as u16;
            self.max_reward_pools = REWARD_ACCOUNT_REWARD_POOLS_MAX_LEN_1 as u8;
            self.data_version = 34;
        }

        if self.data_version == 34 {
            self.get_rewards_iter_mut()
                .for_each(|reward| reward.claimable = 0);
            // previous field:
            // // bit 0: custom contribution accrual rate enabled
            // // bit 1 (deprecated): is closed
            // // bit 2 (deprecated): has holder? (not provided for default holder (fragmetric))
            // reward_pool_bitmap: u8,
            self.get_reward_pools_iter_mut()
                .for_each(|pool| pool.custom_contribution_accrual_rate_enabled &= 1);

            (self.reserve_account, self.reserve_account_bump) =
                Pubkey::find_program_address(&self.get_reserve_account_seed_phrase(), &crate::ID);

            self.data_version = 35;
        }

        require_eq!(self.data_version, REWARD_ACCOUNT_CURRENT_VERSION);

        Ok(())
    }

    #[inline(always)]
    pub(super) fn initialize(&mut self, bump: u8, receipt_token_mint: Pubkey) -> Result<()> {
        self.migrate(bump, receipt_token_mint)
    }

    #[inline(always)]
    pub(super) fn update_if_needed(&mut self, receipt_token_mint: Pubkey) -> Result<()> {
        self.migrate(self.bump, receipt_token_mint)
    }

    #[inline(always)]
    pub fn is_latest_version(&self) -> bool {
        self.data_version == REWARD_ACCOUNT_CURRENT_VERSION
    }

    pub const RESERVE_SEED: &'static [u8] = b"reward_reserve";

    #[inline(always)]
    fn get_reserve_account_seed_phrase(&self) -> [&[u8]; 2] {
        [Self::RESERVE_SEED, self.receipt_token_mint.as_ref()]
    }

    pub(super) fn get_reserve_account_seeds(&self) -> [&[u8]; 3] {
        let mut seeds = <[_; 3]>::default();
        seeds[..2].copy_from_slice(&self.get_reserve_account_seed_phrase());
        seeds[2] = std::slice::from_ref(&self.reserve_account_bump);
        seeds
    }

    pub(super) fn get_reserve_account_address(&self) -> Result<Pubkey> {
        Ok(
            Pubkey::create_program_address(&self.get_reserve_account_seeds(), &crate::ID)
                .map_err(|_| ProgramError::InvalidSeeds)?,
        )
    }

    pub(super) fn find_reward_token_reserve_account_address(
        &self,
        reward_id: u16,
    ) -> Result<Pubkey> {
        let reward = self.get_reward(reward_id)?;
        Ok(
            spl_associated_token_account::get_associated_token_address_with_program_id(
                &self.get_reserve_account_address()?,
                &reward.mint,
                &reward.program,
            ),
        )
    }

    #[inline(always)]
    pub(super) fn get_rewards_iter(&self) -> impl Iterator<Item = &Reward> {
        self.rewards_1[..self.num_rewards as usize].iter()
    }

    #[inline(always)]
    pub(super) fn get_rewards_iter_mut(&mut self) -> impl Iterator<Item = &mut Reward> {
        self.rewards_1[..self.num_rewards as usize].iter_mut()
    }

    pub(super) fn get_reward(&self, id: u16) -> Result<&Reward> {
        self.rewards_1[..self.num_rewards as usize]
            .get(id as usize)
            .ok_or_else(|| error!(ErrorCode::RewardNotFoundError))
    }

    pub(super) fn add_reward(
        &mut self,
        name: String,
        description: String,
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
    ) -> Result<()> {
        if self
            .get_rewards_iter()
            .any(|reward| reward.get_name() == Ok(name.trim_matches('\0')))
        {
            err!(ErrorCode::RewardAlreadyExistingRewardError)?;
        }

        require_gt!(
            self.max_rewards,
            self.num_rewards,
            ErrorCode::RewardExceededMaxRewardsError,
        );

        self.rewards_1[self.num_rewards as usize].initialize(
            self.num_rewards,
            name,
            description,
            mint,
            program,
            decimals,
        )?;
        self.num_rewards += 1;

        Ok(())
    }

    #[inline(always)]
    pub(super) fn get_reward_pools_iter(&self) -> impl Iterator<Item = &RewardPool> {
        self.reward_pools_1[..self.num_reward_pools as usize].iter()
    }

    #[inline(always)]
    pub(super) fn get_reward_pools_iter_mut(&mut self) -> impl Iterator<Item = &mut RewardPool> {
        self.reward_pools_1[..self.num_reward_pools as usize].iter_mut()
    }

    pub(super) fn get_reward_pool_mut(&mut self, id: u8) -> Result<&mut RewardPool> {
        self.reward_pools_1[..self.num_reward_pools as usize]
            .get_mut(id as usize)
            .ok_or_else(|| error!(ErrorCode::RewardPoolNotFoundError))
    }

    pub(super) fn add_reward_pool(
        &mut self,
        name: String,
        custom_contribution_accrual_rate_enabled: bool,
        current_slot: u64,
    ) -> Result<()> {
        if self
            .get_reward_pools_iter()
            .any(|pool| pool.get_name() == Ok(name.trim_matches('\0')))
        {
            err!(ErrorCode::RewardAlreadyExistingPoolError)?
        }

        require_gt!(
            self.max_reward_pools,
            self.num_reward_pools,
            ErrorCode::RewardExceededMaxRewardPoolsError,
        );

        self.reward_pools_1[self.num_reward_pools as usize].initialize(
            self.num_reward_pools,
            name,
            custom_contribution_accrual_rate_enabled,
            current_slot,
        )?;
        self.num_reward_pools += 1;

        Ok(())
    }

    pub(super) fn update_reward_pools(&mut self, current_slot: u64) {
        self.get_reward_pools_iter_mut().for_each(|pool| {
            pool.update_reward_settlements(current_slot);
        });
    }

    pub(super) fn update_reward_pools_token_allocation(
        &mut self,
        amount: u64,
        contribution_accrual_rate: Option<u16>,
        from: Option<&mut UserRewardAccount>,
        to: Option<&mut UserRewardAccount>,
        current_slot: u64,
    ) -> Result<()> {
        // Contribution accrual rate is only allowed for deposits
        if contribution_accrual_rate.is_some() && !(from.is_none() && to.is_some()) {
            err!(ErrorCode::RewardInvalidTransferArgsException)?
        }

        if let Some(from) = from {
            // back-fill not existing pools
            from.backfill_not_existing_pools(self)?;
            for reward_pool in self.get_reward_pools_iter_mut() {
                let user_reward_pool = from.get_user_reward_pool_mut(reward_pool.id)?;
                let deltas = vec![TokenAllocatedAmountDelta::new_negative(amount)];

                let effective_deltas = user_reward_pool.update_token_allocated_amount(
                    reward_pool,
                    deltas,
                    current_slot,
                )?;
                reward_pool.update_token_allocated_amount(effective_deltas, current_slot)?;
            }
        }

        if let Some(to) = to {
            // back-fill not existing pools
            to.backfill_not_existing_pools(self)?;
            for reward_pool in self.get_reward_pools_iter_mut() {
                let user_reward_pool = to.get_user_reward_pool_mut(reward_pool.id)?;
                let effective_contribution_accrual_rate =
                    (reward_pool.custom_contribution_accrual_rate_enabled == 1)
                        .then_some(contribution_accrual_rate)
                        .flatten();
                let deltas = vec![TokenAllocatedAmountDelta::new_positive(
                    effective_contribution_accrual_rate,
                    amount,
                )];
                let effective_deltas = user_reward_pool.update_token_allocated_amount(
                    reward_pool,
                    deltas,
                    current_slot,
                )?;
                reward_pool.update_token_allocated_amount(effective_deltas, current_slot)?;
            }
        }

        Ok(())
    }
}
