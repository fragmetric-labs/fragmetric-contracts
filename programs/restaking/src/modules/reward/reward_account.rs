use anchor_lang::prelude::*;
use anchor_spl::associated_token::spl_associated_token_account;

use crate::errors::ErrorCode;
use crate::utils::{PDASeeds, ZeroCopyHeader};

use super::*;

#[constant]
/// ## Version History
/// * v34: Initial Version (Data Size = 342072 ~= 335KB)
/// * v35: remove holder (Data Size = 342072 ~= 335KB)
pub const REWARD_ACCOUNT_CURRENT_VERSION: u16 = 35;
const REWARD_ACCOUNT_REWARDS_MAX_LEN_1: usize = 16;

#[account(zero_copy)]
#[repr(C)]
pub struct RewardAccount {
    data_version: u16,
    bump: u8,
    pub receipt_token_mint: Pubkey,
    reserve_account_bump: u8,

    max_rewards: u16,
    _padding: [u8; 2],

    num_rewards: u16,
    _padding2: [u8; 6],

    // informative
    reserve_account: Pubkey,

    _reserved: [u8; 2592],

    rewards_1: [Reward; REWARD_ACCOUNT_REWARDS_MAX_LEN_1],

    pub(super) base_reward_pool: RewardPool,
    pub(super) bonus_reward_pool: RewardPool,

    _reserved2: [u8; 83440],
    _reserved3: [u8; 83440],
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
    fn migrate(&mut self, bump: u8, receipt_token_mint: Pubkey, current_slot: u64) -> Result<()> {
        if self.data_version == 0 {
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
            self.max_rewards = REWARD_ACCOUNT_REWARDS_MAX_LEN_1 as u16;
            self.base_reward_pool
                .initialize(BASE_REWARD_POOL_ID, false, current_slot)?;
            self.bonus_reward_pool
                .initialize(BONUS_REWARD_POOL_ID, true, current_slot)?;
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
            self.base_reward_pool
                .custom_contribution_accrual_rate_enabled &= 1;
            self.bonus_reward_pool
                .custom_contribution_accrual_rate_enabled &= 1;

            (self.reserve_account, self.reserve_account_bump) =
                Pubkey::find_program_address(&self.get_reserve_account_seed_phrase(), &crate::ID);

            // Clear dirty bits from previous fields, for future use
            self._padding[0] = 0; // max_reward_pools
            self._padding[1] = 0; // num_holders
            self._padding2[0] = 0; // num_reward_pools

            self.data_version = 35;
        }

        require_eq!(self.data_version, REWARD_ACCOUNT_CURRENT_VERSION);

        Ok(())
    }

    #[inline(always)]
    pub(super) fn initialize(
        &mut self,
        bump: u8,
        receipt_token_mint: Pubkey,
        current_slot: u64,
    ) -> Result<()> {
        self.migrate(bump, receipt_token_mint, current_slot)
    }

    #[inline(always)]
    pub(super) fn update_if_needed(
        &mut self,
        receipt_token_mint: Pubkey,
        current_slot: u64,
    ) -> Result<()> {
        self.migrate(self.bump, receipt_token_mint, current_slot)
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

    #[inline(always)]
    pub(super) fn get_rewards_iter(&self) -> impl Iterator<Item = &Reward> {
        self.rewards_1[..self.num_rewards as usize].iter()
    }

    #[inline(always)]
    pub(super) fn get_rewards_iter_mut(&mut self) -> impl Iterator<Item = &mut Reward> {
        self.rewards_1[..self.num_rewards as usize].iter_mut()
    }

    pub(super) fn get_reward(&self, reward_id: u16) -> Result<&Reward> {
        self.rewards_1[..self.num_rewards as usize]
            .get(reward_id as usize)
            .ok_or_else(|| error!(ErrorCode::RewardNotFoundError))
    }

    pub(super) fn get_reward_mut(&mut self, reward_id: u16) -> Result<&mut Reward> {
        self.rewards_1[..self.num_rewards as usize]
            .get_mut(reward_id as usize)
            .ok_or_else(|| error!(ErrorCode::RewardNotFoundError))
    }

    pub(super) fn get_reward_id(&self, reward_token_mint: &Pubkey) -> Result<u16> {
        self.get_rewards_iter()
            .find_map(|reward| (reward.mint == *reward_token_mint).then_some(reward.id))
            .ok_or_else(|| error!(ErrorCode::RewardNotFoundError))
    }

    /// returns reward id
    pub(super) fn add_reward(
        &mut self,
        name: String,
        description: String,
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        claimable: bool,
    ) -> Result<u16> {
        if self
            .get_rewards_iter()
            .any(|reward| reward.get_name() == Ok(name.trim_matches('\0')) || reward.mint == mint)
        {
            err!(ErrorCode::RewardAlreadyExistingRewardError)?;
        }

        require_gt!(
            self.max_rewards,
            self.num_rewards,
            ErrorCode::RewardExceededMaxRewardsError,
        );

        let reward_id = self.num_rewards;
        self.rewards_1[reward_id as usize].initialize(
            reward_id,
            name,
            description,
            mint,
            program,
            decimals,
            claimable,
        )?;
        self.num_rewards += 1;

        Ok(reward_id)
    }

    pub(super) fn update_reward(
        &mut self,
        reward_id: u16,
        new_mint: Option<Pubkey>,
        new_program: Option<Pubkey>,
        new_decimals: Option<u8>,
        claimable: bool,
    ) -> Result<()> {
        // New mint should not be duplicated with other reward mints.
        if new_mint.as_ref().is_some_and(|new_mint| {
            self.get_rewards_iter()
                .any(|reward| reward.id != reward_id && reward.mint == *new_mint)
        }) {
            err!(ErrorCode::RewardAlreadyExistingRewardError)?
        }

        self.get_reward_mut(reward_id)?
            .set_reward_token(new_mint, new_program, new_decimals)?
            .set_claimable(claimable)?;

        Ok(())
    }

    pub(super) fn settle_reward(
        &mut self,
        reward_id: u16,
        is_bonus_pool: bool,
        amount: u64,
        current_slot: u64,
    ) -> Result<()> {
        self.get_reward_pool_mut(is_bonus_pool)
            .settle_reward(reward_id, amount, current_slot)?;

        Ok(())
    }

    #[inline(always)]
    pub(super) fn get_reward_pools_iter_mut(&mut self) -> impl Iterator<Item = &mut RewardPool> {
        [&mut self.base_reward_pool, &mut self.bonus_reward_pool].into_iter()
    }

    pub(super) fn get_reward_pool_mut(&mut self, is_bonus_pool: bool) -> &mut RewardPool {
        if !is_bonus_pool {
            &mut self.base_reward_pool
        } else {
            &mut self.bonus_reward_pool
        }
    }

    pub(super) fn get_unclaimed_reward_amount(&self, reward_id: u16) -> u64 {
        let base_pool_unclaimed_amount =
            self.base_reward_pool.get_unclaimed_reward_amount(reward_id);
        let bonus_pool_unclaimed_amount = self
            .bonus_reward_pool
            .get_unclaimed_reward_amount(reward_id);

        base_pool_unclaimed_amount + bonus_pool_unclaimed_amount
    }

    /// Updates the contribution of the pools and clear stale settlement blocks.
    ///
    /// this operation is idempotent
    pub(super) fn update_reward_pools(&mut self, current_slot: u64) {
        self.base_reward_pool.update_reward_pool(current_slot);
        self.bonus_reward_pool.update_reward_pool(current_slot);
    }
}
