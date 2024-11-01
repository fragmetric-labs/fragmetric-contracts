use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::errors::ErrorCode;
use crate::utils::{PDASeeds, ZeroCopyHeader};

use super::*;

#[constant]
/// ## Version History
/// * v34: Initial Version (Data Size = 342064 ~= 335KB)
pub const REWARD_ACCOUNT_CURRENT_VERSION: u16 = 34;
const REWARD_ACCOUNT_HOLDERS_MAX_LEN_1: usize = 4;
const REWARD_ACCOUNT_REWARDS_MAX_LEN_1: usize = 16;
const REWARD_ACCOUNT_REWARD_POOLS_MAX_LEN_1: usize = 4;

#[account(zero_copy)]
#[repr(C)]
pub struct RewardAccount {
    data_version: u16,
    bump: u8,
    pub receipt_token_mint: Pubkey,
    max_holders: u8,
    max_rewards: u16,
    max_reward_pools: u8,
    num_holders: u8,
    num_rewards: u16,
    num_reward_pools: u8,
    _padding: [u8; 5],

    holders_1: [Holder; REWARD_ACCOUNT_HOLDERS_MAX_LEN_1],
    rewards_1: [Reward; REWARD_ACCOUNT_REWARDS_MAX_LEN_1],
    reward_pools_1: [RewardPool; REWARD_ACCOUNT_REWARD_POOLS_MAX_LEN_1],
}

impl PDASeeds<2> for RewardAccount {
    const SEED: &'static [u8] = b"reward";

    fn get_seeds(&self) -> [&[u8]; 2] {
        [Self::SEED, self.receipt_token_mint.as_ref()]
    }

    fn get_bump_ref(&self) -> &u8 {
        &self.bump
    }
}

impl ZeroCopyHeader for RewardAccount {
    fn get_bump_offset() -> usize {
        2
    }
}

impl RewardAccount {
    pub(super) fn initialize(&mut self, bump: u8, receipt_token_mint: Pubkey) {
        if self.data_version == 0 {
            self.data_version = 34;
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
            self.max_holders = REWARD_ACCOUNT_HOLDERS_MAX_LEN_1 as u8;
            self.max_rewards = REWARD_ACCOUNT_REWARDS_MAX_LEN_1 as u16;
            self.max_reward_pools = REWARD_ACCOUNT_REWARD_POOLS_MAX_LEN_1 as u8;
        }

        // if self.data_version == 34 {
        //     self.data_version = 43;
        //     self.max_reward_pools += REWARD_ACCOUNT_REWARD_POOLS_MAX_LEN_2 as u8;
        // }
    }

    #[inline(always)]
    pub(super) fn update_if_needed(&mut self, receipt_token_mint: Pubkey) {
        self.initialize(self.bump, receipt_token_mint);
    }

    #[inline(always)]
    pub fn is_latest_version(&self) -> bool {
        self.data_version == REWARD_ACCOUNT_CURRENT_VERSION
    }

    pub(super) fn settle_reward(
        &mut self,
        reward_pool_id: u8,
        reward_id: u16,
        amount: u64,
        current_slot: u64,
    ) -> Result<()> {
        require_gt!(self.num_rewards, reward_id, ErrorCode::RewardNotFoundError);

        self.get_reward_pool_mut(reward_pool_id)?
            .settle_reward(reward_id, amount, current_slot)
    }

    pub(super) fn add_new_holder(
        &mut self,
        name: String,
        description: String,
        pubkeys: Vec<Pubkey>,
    ) -> Result<()> {
        for holder in self.get_holders_iter() {
            require_neq!(
                holder.get_name()?,
                name.trim_matches('\0'),
                ErrorCode::RewardAlreadyExistingHolderError
            );
        }

        require_gt!(
            self.max_holders,
            self.num_holders,
            ErrorCode::RewardExceededMaxHoldersException,
        );

        let holder = &mut self.holders_1[self.num_holders as usize];
        holder.initialize(self.num_holders, name, description, &pubkeys)?;
        self.num_holders += 1;

        Ok(())
    }

    pub(super) fn add_new_reward(
        &mut self,
        name: String,
        description: String,
        reward_type: RewardType,
    ) -> Result<()> {
        for reward in self.get_rewards_iter() {
            require_neq!(
                reward.get_name()?,
                name.trim_matches('\0'),
                ErrorCode::RewardAlreadyExistingRewardError
            );
        }

        require_gt!(
            self.max_rewards,
            self.num_rewards,
            ErrorCode::RewardExceededMaxRewardsException,
        );

        let reward = &mut self.rewards_1[self.num_rewards as usize];
        reward.initialize(self.num_rewards, name, description, reward_type)?;
        self.num_rewards += 1;

        Ok(())
    }

    pub(super) fn add_new_reward_pool(
        &mut self,
        name: String,
        holder_id: Option<u8>,
        custom_contribution_accrual_rate_enabled: bool,
        current_slot: u64,
    ) -> Result<()> {
        if let Some(id) = holder_id {
            require_gt!(self.num_holders, id, ErrorCode::RewardHolderNotFoundError);
        }

        if self.get_reward_pools_iter().any(|p| {
            (p.get_holder_id() == holder_id
                && p.is_custom_contribution_accrual_rate_enabled()
                == custom_contribution_accrual_rate_enabled)
                || p.get_name() == Ok(name.trim_matches('\0'))
        }) {
            err!(ErrorCode::RewardAlreadyExistingPoolError)?
        }

        require_gt!(
            self.max_reward_pools,
            self.num_reward_pools,
            ErrorCode::RewardExceededMaxRewardPoolsException,
        );

        let pool = &mut self.reward_pools_1[self.num_reward_pools as usize];
        pool.initialize(
            self.num_reward_pools,
            name,
            holder_id,
            custom_contribution_accrual_rate_enabled,
            current_slot,
        )?;
        self.num_reward_pools += 1;

        Ok(())
    }

    pub(super) fn close_reward_pool(
        &mut self,
        reward_pool_id: u8,
        current_slot: u64,
    ) -> Result<()> {
        let reward_pool = self.get_reward_pool_mut(reward_pool_id)?;

        // Cannot close reward pool without holder
        match reward_pool.get_holder_id() {
            Some(_) => reward_pool.close(current_slot),
            None => err!(ErrorCode::RewardPoolCloseConditionError)?,
        }
    }

    pub(super) fn update_reward_pools(&mut self, current_slot: u64) -> Result<()> {
        for reward_pool in self.get_reward_pools_iter_mut() {
            let updated_slot = reward_pool.get_closed_slot().unwrap_or(current_slot);
            reward_pool.update_contribution(updated_slot)?;
            for reward_settlement in reward_pool.get_reward_settlements_iter_mut() {
                reward_settlement.clear_stale_settlement_blocks()?;
            }
        }

        Ok(())
    }

    pub(super) fn update_user_reward_pools(
        &mut self,
        user: &mut UserRewardAccount,
        current_slot: u64,
    ) -> Result<()> {
        user.backfill_not_existing_pools(self.get_reward_pools_iter())?;

        user.get_user_reward_pools_iter_mut()
            .zip(self.get_reward_pools_iter_mut())
            .try_for_each(|(user_reward_pool, reward_pool)| {
                user_reward_pool.update(reward_pool, vec![], current_slot)?;
                Ok::<(), Error>(())
            })?;

        Ok(())
    }

    pub(super) fn update_reward_pools_token_allocation(
        &mut self,
        receipt_token_mint: Pubkey,
        amount: u64,
        contribution_accrual_rate: Option<u8>,
        from: Option<&mut UserRewardAccount>,
        to: Option<&mut UserRewardAccount>,
        current_slot: u64,
    ) -> Result<()> {
        if from.is_none() && to.is_none() || to.is_none() && contribution_accrual_rate.is_some() {
            err!(ErrorCode::RewardInvalidTransferArgsException)?
        }

        if let Some(from) = from {
            // back-fill not existing pools
            from.backfill_not_existing_pools(self.get_reward_pools_iter())?;
            // find "from user" related reward pools
            for reward_pool in self.get_related_pools(&from.user, receipt_token_mint)? {
                let user_reward_pool = from.get_user_reward_pool_mut(reward_pool.get_id())?;
                let deltas = vec![TokenAllocatedAmountDelta::new_negative(amount)];

                let effective_deltas =
                    user_reward_pool.update(reward_pool, deltas, current_slot)?;
                reward_pool.update(effective_deltas, current_slot)?;
            }
        }

        if let Some(to) = to {
            // back-fill not existing pools
            to.backfill_not_existing_pools(self.get_reward_pools_iter())?;
            // find "to user" related reward pools
            for reward_pool in self.get_related_pools(&to.user, receipt_token_mint)? {
                let user_reward_pool = to.get_user_reward_pool_mut(reward_pool.get_id())?;
                let effective_contribution_accrual_rate = reward_pool
                    .is_custom_contribution_accrual_rate_enabled()
                    .then_some(contribution_accrual_rate)
                    .flatten();
                let deltas = vec![TokenAllocatedAmountDelta::new_positive(
                    effective_contribution_accrual_rate,
                    amount,
                )];
                let effective_deltas =
                    user_reward_pool.update(reward_pool, deltas, current_slot)?;
                reward_pool.update(effective_deltas, current_slot)?;
            }
        }

        Ok(())
    }

    fn get_related_pools(
        &mut self,
        user: &Pubkey,
        receipt_token_mint: Pubkey,
    ) -> Result<Vec<&mut RewardPool>> {
        if self.receipt_token_mint != receipt_token_mint {
            return Err(ErrorCode::RewardInvalidPoolAccessException)?;
        }

        let user_related_holders_ids = self
            .get_holders_iter()
            .filter_map(|holder| holder.has_pubkey(user).then_some(holder.get_id()))
            .collect::<Vec<_>>();

        let reward_pools = self.get_reward_pools_iter_mut();
        // split into base / holder-specific pools
        let (base, holder_specific) =
            reward_pools.partition::<Vec<_>, _>(|p| p.get_holder_id().is_none());

        // base pool should exist at least one
        if base.is_empty() {
            err!(ErrorCode::RewardInvalidPoolConfigurationException)?
        }

        // first try to find within holder specific pools
        let mut related = holder_specific
            .into_iter()
            .filter(|p| {
                // SAFE: partitioned by `holder_id.is_none()`
                user_related_holders_ids.contains(&p.get_holder_id().unwrap())
            })
            .collect::<Vec<_>>();

        // otherwise falls back to base pools
        if related.is_empty() {
            related = base;
        }

        Ok(related)
    }

    /// How to integrate multiple fields into a single array slice or whatever...
    /// You may change the return type if needed
    #[inline(always)]
    fn get_holders(&self) -> &[Holder] {
        &self.holders_1[..self.num_holders as usize]
    }

    #[inline(always)]
    fn get_holders_iter(&self) -> impl Iterator<Item=&Holder> {
        self.get_holders().iter()
    }

    /// How to integrate multiple fields into a single array slice or whatever...
    /// You may change the return type if needed
    #[inline(always)]
    fn get_rewards(&self) -> &[Reward] {
        &self.rewards_1[..self.num_rewards as usize]
    }

    #[inline(always)]
    fn get_rewards_iter(&self) -> impl Iterator<Item=&Reward> {
        self.get_rewards().iter()
    }

    /// How to integrate multiple fields into a single array slice or whatever...
    /// You may change the return type if needed
    #[inline(always)]
    fn get_reward_pools(&self) -> &[RewardPool] {
        &self.reward_pools_1[..self.num_reward_pools as usize]
    }

    /// How to integrate multiple fields into a single array slice or whatever...
    /// You may change the return type if needed
    #[inline(always)]
    fn get_reward_pools_mut(&mut self) -> &mut [RewardPool] {
        &mut self.reward_pools_1[..self.num_reward_pools as usize]
    }

    #[inline(always)]
    fn get_reward_pools_iter(&self) -> impl Iterator<Item=&RewardPool> {
        self.get_reward_pools().iter()
    }

    #[inline(always)]
    fn get_reward_pools_iter_mut(&mut self) -> impl Iterator<Item=&mut RewardPool> {
        self.get_reward_pools_mut().iter_mut()
    }

    fn get_reward_pool_mut(&mut self, id: u8) -> Result<&mut RewardPool> {
        self.get_reward_pools_mut()
            .get_mut(id as usize)
            .ok_or_else(|| error!(ErrorCode::RewardPoolNotFoundError))
    }
}

const REWARD_POOL_NAME_MAX_LEN: usize = 14;
const REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_1: usize = 16;
// const REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_2: usize = 8;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct RewardPool {
    /// ID is determined by reward account.
    id: u8,
    name: [u8; REWARD_POOL_NAME_MAX_LEN],

    // bit 0: custom contribution accrual rate enabled?
    // bit 1: is closed?
    // bit 2: has holder? (not provided for default holder (fragmetric))
    reward_pool_bitmap: u8,

    token_allocated_amount: TokenAllocatedAmount,
    contribution: u128,

    initial_slot: u64,
    updated_slot: u64,
    closed_slot: u64,

    holder_id: u8,
    num_reward_settlements: u8,
    _padding: [u8; 6],

    _reserved: [u64; 32], // 256 byte

    reward_settlements_1: [RewardSettlement; REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_1],
}

// When you want to extend reward settlements at update v3...
// ```
// pub struct RewardPoolExtV3 {
//     id: u8,
//     num_reward_settlements: u8,
//     _padding: [u8; 14],
//     reward_settlements_2: [RewardSettlement; REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_2],
// }
// ```
// And add new field reward_pools_1_ext_v3: [RewardPoolExtV3; REWARD_ACCOUNT_REWARD_POOLS_MAX_LEN_1] to reward account.

impl RewardPool {
    const CUSTOM_CONTRIBUTION_ACCRUAL_RATE_ENABLED_BIT: u8 = 1 << 0;
    const IS_CLOSED_BIT: u8 = 1 << 1;
    const HAS_HOLDER_BIT: u8 = 1 << 2;

    fn initialize(
        &mut self,
        id: u8,
        name: String,
        holder_id: Option<u8>,
        custom_contribution_accrual_rate_enabled: bool,
        current_slot: u64,
    ) -> Result<()> {
        require_gte!(
            REWARD_POOL_NAME_MAX_LEN,
            name.len(),
            ErrorCode::RewardInvalidMetadataNameLengthError
        );

        self.id = id;
        self.name[..name.len()].copy_from_slice(name.as_bytes());
        self.reward_pool_bitmap &= 0; // reset
        if custom_contribution_accrual_rate_enabled {
            self.reward_pool_bitmap |= Self::CUSTOM_CONTRIBUTION_ACCRUAL_RATE_ENABLED_BIT;
        }
        if holder_id.is_some() {
            self.reward_pool_bitmap |= Self::HAS_HOLDER_BIT;
        }
        self.token_allocated_amount = TokenAllocatedAmount::zeroed();
        self.contribution = 0;
        self.initial_slot = current_slot;
        self.updated_slot = current_slot;
        self.closed_slot = 0;
        self.holder_id = holder_id.unwrap_or_default();
        self.num_reward_settlements = 0;

        Ok(())
    }

    #[inline(always)]
    pub(super) fn get_id(&self) -> u8 {
        self.id
    }

    fn get_name(&self) -> Result<&str> {
        Ok(std::str::from_utf8(&self.name)
            .map_err(|_| crate::errors::ErrorCode::DecodeInvalidUtf8FormatException)?
            .trim_matches('\0'))
    }

    #[inline(always)]
    fn is_custom_contribution_accrual_rate_enabled(&self) -> bool {
        self.reward_pool_bitmap & Self::CUSTOM_CONTRIBUTION_ACCRUAL_RATE_ENABLED_BIT > 0
    }

    #[inline(always)]
    pub(super) fn get_initial_slot(&self) -> u64 {
        self.initial_slot
    }

    #[inline(always)]
    pub(super) fn get_closed_slot(&self) -> Option<u64> {
        self.is_closed().then_some(self.closed_slot)
    }

    #[inline(always)]
    fn is_closed(&self) -> bool {
        self.reward_pool_bitmap & Self::IS_CLOSED_BIT > 0
    }

    fn set_closed(&mut self, closed_slot: u64) {
        self.reward_pool_bitmap |= Self::IS_CLOSED_BIT;
        self.closed_slot = closed_slot;
    }

    #[inline(always)]
    fn get_holder_id(&self) -> Option<u8> {
        (self.reward_pool_bitmap & Self::HAS_HOLDER_BIT > 0).then_some(self.holder_id)
    }

    fn add_new_reward_settlement(
        &mut self,
        reward_id: u16,
        current_slot: u64,
    ) -> Result<&mut RewardSettlement> {
        require_gt!(
            REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_1,
            self.num_reward_settlements as usize,
            ErrorCode::RewardExceededMaxRewardPoolsException,
        );

        let settlement = &mut self.reward_settlements_1[self.num_reward_settlements as usize];
        settlement.initialize(reward_id, self.id, self.initial_slot, current_slot);
        self.num_reward_settlements += 1;

        Ok(settlement)
    }

    /// How to integrate multiple fields into a single array slice or whatever...
    /// You may change the return type if needed
    #[inline(always)]
    fn get_reward_settlements_mut(&mut self) -> &mut [RewardSettlement] {
        &mut self.reward_settlements_1[..self.num_reward_settlements as usize]
    }

    #[inline(always)]
    pub(super) fn get_reward_settlements_iter_mut(
        &mut self,
    ) -> impl Iterator<Item=&mut RewardSettlement> {
        self.get_reward_settlements_mut().iter_mut()
    }

    fn get_reward_settlement_mut(&mut self, reward_id: u16) -> Option<&mut RewardSettlement> {
        self.get_reward_settlements_iter_mut()
            .find(|s| s.get_reward_id() == reward_id)
    }

    fn settle_reward(&mut self, reward_id: u16, amount: u64, current_slot: u64) -> Result<()> {
        if self.is_closed() {
            err!(ErrorCode::RewardPoolClosedError)?;
        }

        // First update contribution
        self.update_contribution(current_slot)?;

        // Find settlement and settle
        let current_reward_pool_contribution = self.contribution;
        let settlement = if let Some(settlement) = self.get_reward_settlement_mut(reward_id) {
            settlement
        } else {
            self.add_new_reward_settlement(reward_id, current_slot)?
        };

        settlement.settle_reward(amount, current_reward_pool_contribution, current_slot)
    }

    /// Updates the contribution of the pool into recent value.
    fn update_contribution(&mut self, updated_slot: u64) -> Result<()> {
        let elapsed_slot = updated_slot
            .checked_sub(self.updated_slot)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        let total_contribution_accrual_rate = self
            .token_allocated_amount
            .get_total_contribution_accrual_rate()?;
        let total_contribution = (elapsed_slot as u128)
            .checked_mul(total_contribution_accrual_rate as u128)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self.contribution = self
            .contribution
            .checked_add(total_contribution)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self.updated_slot = updated_slot;

        Ok(())
    }

    fn update(
        &mut self,
        deltas: Vec<TokenAllocatedAmountDelta>,
        current_slot: u64,
    ) -> Result<Vec<TokenAllocatedAmountDelta>> {
        // First update contribution
        let updated_slot = self.get_closed_slot().unwrap_or(current_slot);
        self.update_contribution(updated_slot)?;

        // Apply deltas
        if !deltas.is_empty() {
            self.token_allocated_amount.update(deltas)
        } else {
            Ok(deltas)
        }
    }

    fn close(&mut self, current_slot: u64) -> Result<()> {
        if self.is_closed() {
            err!(ErrorCode::RewardPoolClosedError)?
        }

        // update contribution as last
        self.update_contribution(current_slot)?;
        self.set_closed(current_slot);

        Ok(())
    }
}
