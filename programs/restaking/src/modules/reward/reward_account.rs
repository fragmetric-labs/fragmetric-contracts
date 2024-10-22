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
    pub max_holders: u8,
    pub max_rewards: u16,
    pub max_reward_pools: u8,
    pub num_holders: u8,
    pub num_rewards: u16,
    pub num_reward_pools: u8,
    _padding: [u8; 5],

    holders_1: [Holder; REWARD_ACCOUNT_HOLDERS_MAX_LEN_1],
    rewards_1: [Reward; REWARD_ACCOUNT_REWARDS_MAX_LEN_1],
    reward_pools_1: [RewardPool; REWARD_ACCOUNT_REWARD_POOLS_MAX_LEN_1],
}

impl PDASeeds<2> for RewardAccount {
    const SEED: &'static [u8] = b"reward";

    fn seeds(&self) -> [&[u8]; 2] {
        [Self::SEED, self.receipt_token_mint.as_ref()]
    }

    fn bump_ref(&self) -> &u8 {
        &self.bump
    }
}

impl ZeroCopyHeader for RewardAccount {
    fn bump_offset() -> usize {
        2
    }
}

impl RewardAccount {
    pub fn initialize(&mut self, bump: u8, receipt_token_mint: Pubkey) {
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

    pub fn update_if_needed(&mut self, receipt_token_mint: Pubkey) {
        self.initialize(self.bump, receipt_token_mint);
    }

    pub fn is_latest_version(&self) -> bool {
        self.data_version == REWARD_ACCOUNT_CURRENT_VERSION
    }

    pub fn allocate_new_holder(&mut self) -> Result<&mut Holder> {
        require_gt!(
            self.max_holders,
            self.num_holders,
            ErrorCode::RewardExceededMaxHoldersException,
        );

        let holder = &mut self.holders_1[self.num_holders as usize];
        holder.set_id(self.num_holders);
        self.num_holders += 1;

        Ok(holder)
    }

    pub fn allocate_new_reward(&mut self) -> Result<&mut Reward> {
        require_gt!(
            self.max_rewards,
            self.num_rewards,
            ErrorCode::RewardExceededMaxRewardsException,
        );

        let reward = &mut self.rewards_1[self.num_rewards as usize];
        reward.set_id(self.num_rewards);
        self.num_rewards += 1;

        Ok(reward)
    }

    pub fn allocate_new_reward_pool(&mut self) -> Result<&mut RewardPool> {
        require_gt!(
            self.max_reward_pools,
            self.num_reward_pools,
            ErrorCode::RewardExceededMaxRewardPoolsException,
        );

        let pool = &mut self.reward_pools_1[self.num_reward_pools as usize];
        pool.set_id(self.num_reward_pools);
        self.num_reward_pools += 1;

        Ok(pool)
    }

    pub fn settle_reward(
        &mut self,
        reward_pool_id: u8,
        reward_id: u16,
        amount: u64,
        current_slot: u64,
    ) -> Result<()> {
        require_gt!(self.num_rewards, reward_id, ErrorCode::RewardNotFoundError);

        self.reward_pool_mut(reward_pool_id)?
            .settle_reward(reward_id, amount, current_slot)
    }

    pub fn add_holder(
        &mut self,
        name: String,
        description: String,
        pubkeys: Vec<Pubkey>,
    ) -> Result<()> {
        for holder in self.holders_iter() {
            require_neq!(
                from_utf8_trim_null(holder.name())?,
                name,
                ErrorCode::RewardAlreadyExistingHolderError
            );
        }

        let holder = self.allocate_new_holder()?;
        holder.initialize(name, description, &pubkeys)?;

        Ok(())
    }

    pub fn add_reward(
        &mut self,
        name: String,
        description: String,
        reward_type: RewardType,
    ) -> Result<()> {
        for reward in self.rewards_iter() {
            require_neq!(
                from_utf8_trim_null(reward.name())?,
                name,
                ErrorCode::RewardAlreadyExistingRewardError
            );
        }

        let reward = self.allocate_new_reward()?;
        reward.initialize(name, description, reward_type)?;

        Ok(())
    }

    pub fn add_reward_pool(
        &mut self,
        name: String,
        holder_id: Option<u8>,
        custom_contribution_accrual_rate_enabled: bool,
        current_slot: u64,
    ) -> Result<()> {
        if let Some(id) = holder_id {
            require_gt!(self.num_holders, id, ErrorCode::RewardHolderNotFoundError);
        }

        if self.reward_pools_iter().any(|p| {
            (p.holder_id() == holder_id
                && p.custom_contribution_accrual_rate_enabled()
                    == custom_contribution_accrual_rate_enabled)
                || from_utf8_trim_null(p.name()).as_ref() == Ok(&name)
        }) {
            err!(ErrorCode::RewardAlreadyExistingPoolError)?
        }

        let reward_pool = self.allocate_new_reward_pool()?;
        reward_pool.initialize(
            name,
            holder_id,
            custom_contribution_accrual_rate_enabled,
            current_slot,
        )?;

        Ok(())
    }

    pub fn close_reward_pool(&mut self, reward_pool_id: u8, current_slot: u64) -> Result<()> {
        let reward_pool = self.reward_pool_mut(reward_pool_id)?;
        match reward_pool.holder_id() {
            None => err!(ErrorCode::RewardPoolCloseConditionError)?,
            Some(_) => reward_pool.close(current_slot),
        }
    }

    pub fn update_reward_pools(&mut self, current_slot: u64) -> Result<()> {
        for reward_pool in self.reward_pools_iter_mut() {
            let updated_slot = reward_pool.closed_slot().unwrap_or(current_slot);
            reward_pool.update_contribution(updated_slot)?;
            for reward_settlement in reward_pool.reward_settlements_iter_mut() {
                reward_settlement.clear_stale_settlement_blocks()?;
            }
        }

        Ok(())
    }

    pub fn update_user_reward_pools(
        &mut self,
        user: &mut UserRewardAccount,
        current_slot: u64,
    ) -> Result<()> {
        user.backfill_not_existing_pools(self.reward_pools_iter())?;

        user.user_reward_pools_iter_mut()
            .zip(self.reward_pools_iter_mut())
            .try_for_each(|(user_reward_pool, reward_pool)| {
                user_reward_pool.update(reward_pool, vec![], current_slot)?;
                Ok::<(), Error>(())
            })?;

        Ok(())
    }

    pub fn update_reward_pools_token_allocation(
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
            from.backfill_not_existing_pools(self.reward_pools_iter())?;
            // find "from user" related reward pools
            for reward_pool in self.get_related_pools(&from.user, receipt_token_mint)? {
                let user_reward_pool = from.user_reward_pool_mut(reward_pool.id())?;
                let deltas = vec![TokenAllocatedAmountDelta {
                    contribution_accrual_rate: None,
                    is_positive: false,
                    amount,
                }];

                let effective_deltas =
                    user_reward_pool.update(reward_pool, deltas, current_slot)?;
                reward_pool.update(effective_deltas, current_slot)?;
            }
        }

        if let Some(to) = to {
            // back-fill not existing pools
            to.backfill_not_existing_pools(self.reward_pools_iter())?;
            // find "to user" related reward pools
            for reward_pool in self.get_related_pools(&to.user, receipt_token_mint)? {
                let user_reward_pool = to.user_reward_pool_mut(reward_pool.id())?;
                let effective_contribution_accrual_rate = reward_pool
                    .custom_contribution_accrual_rate_enabled()
                    .then_some(contribution_accrual_rate)
                    .flatten();
                let deltas = vec![TokenAllocatedAmountDelta {
                    contribution_accrual_rate: effective_contribution_accrual_rate,
                    is_positive: true,
                    amount,
                }];
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

        let (holders_ref, reward_pools) = self.holders_ref_and_reward_pools_iter_mut();
        // split into base / holder-specific pools
        let (base, holder_specific) =
            reward_pools.partition::<Vec<_>, _>(|p| p.holder_id().is_none());

        // base pool should exist at least one
        if base.is_empty() {
            err!(ErrorCode::RewardInvalidPoolConfigurationException)?
        }

        // first try to find within holder specific pools
        let mut related = holder_specific
            .into_iter()
            .filter(|p| {
                // SAFE: partitioned by `holder_id.is_none()`
                // SAFE: always checks holder existence when adding reward pool
                holders_ref[p.holder_id().unwrap() as usize].has_pubkey(user)
            })
            .collect::<Vec<_>>();

        // otherwise falls back to base pools
        if related.is_empty() {
            related = base;
        }

        Ok(related)
    }

    /// Auxillary method to breaks down a mutable borrow into separate borrows over fields
    fn array_refs_mut(&mut self) -> (&mut [Holder], &mut [Reward], &mut [RewardPool]) {
        let holders_mut = &mut self.holders_1[..self.num_holders as usize];
        let rewards_mut = &mut self.rewards_1[..self.num_rewards as usize];
        let reward_pools_mut = &mut self.reward_pools_1[..self.num_reward_pools as usize];
        (holders_mut, rewards_mut, reward_pools_mut)
    }

    /// Auxillary method to breaks down a mutable borrow into separate borrows over fields
    /// TODO Do not expose array slice type to public
    pub fn holders_ref_and_reward_pools_iter_mut(
        &mut self,
    ) -> (&[Holder], impl Iterator<Item = &mut RewardPool>) {
        let (holders_ref, _, reward_pools_mut) = self.array_refs_mut();
        (holders_ref, reward_pools_mut.iter_mut())
    }

    /// How to integrate multiple fields into a single array slice or whatever...
    /// You may change the return type if needed
    fn holders_ref(&self) -> &[Holder] {
        &self.holders_1[..self.num_holders as usize]
    }

    pub fn holders_iter(&self) -> impl Iterator<Item = &Holder> {
        self.holders_ref().iter()
    }

    /// How to integrate multiple fields into a single array slice or whatever...
    /// You may change the return type if needed
    fn rewards_ref(&self) -> &[Reward] {
        &self.rewards_1[..self.num_rewards as usize]
    }

    pub fn rewards_iter(&self) -> impl Iterator<Item = &Reward> {
        self.rewards_ref().iter()
    }

    /// How to integrate multiple fields into a single array slice or whatever...
    /// You may change the return type if needed
    fn reward_pools_ref(&self) -> &[RewardPool] {
        &self.reward_pools_1[..self.num_reward_pools as usize]
    }

    /// How to integrate multiple fields into a single array slice or whatever...
    /// You may change the return type if needed
    fn reward_pools_mut(&mut self) -> &mut [RewardPool] {
        self.array_refs_mut().2
    }

    pub fn reward_pools_iter(&self) -> impl Iterator<Item = &RewardPool> {
        self.reward_pools_ref().iter()
    }

    pub fn reward_pools_iter_mut(&mut self) -> impl Iterator<Item = &mut RewardPool> {
        self.reward_pools_mut().iter_mut()
    }

    pub fn reward_pool_mut(&mut self, id: u8) -> Result<&mut RewardPool> {
        self.reward_pools_mut()
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

    pub token_allocated_amount: TokenAllocatedAmount,
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

    pub fn initialize(
        &mut self,
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

    pub fn id(&self) -> u8 {
        self.id
    }

    fn set_id(&mut self, id: u8) {
        self.id = id;
    }

    pub fn name(&self) -> &[u8] {
        &self.name
    }

    pub fn custom_contribution_accrual_rate_enabled(&self) -> bool {
        self.reward_pool_bitmap & Self::CUSTOM_CONTRIBUTION_ACCRUAL_RATE_ENABLED_BIT > 0
    }

    pub fn contribution(&self) -> u128 {
        self.contribution
    }

    pub fn add_contribution(&mut self, contribution: u128, updated_slot: u64) -> Result<()> {
        self.contribution = self
            .contribution
            .checked_add(contribution)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self.updated_slot = updated_slot;

        Ok(())
    }

    pub fn initial_slot(&self) -> u64 {
        self.initial_slot
    }

    pub fn updated_slot(&self) -> u64 {
        self.updated_slot
    }

    pub fn closed_slot(&self) -> Option<u64> {
        self.is_closed().then_some(self.closed_slot)
    }

    pub fn is_closed(&self) -> bool {
        self.reward_pool_bitmap & Self::IS_CLOSED_BIT > 0
    }

    pub fn set_closed(&mut self, closed_slot: u64) {
        self.reward_pool_bitmap |= Self::IS_CLOSED_BIT;
        self.closed_slot = closed_slot;
    }

    pub fn holder_id(&self) -> Option<u8> {
        (self.reward_pool_bitmap & Self::HAS_HOLDER_BIT > 0).then_some(self.holder_id)
    }

    pub fn allocate_new_reward_settlement(&mut self) -> Result<&mut RewardSettlement> {
        require_gt!(
            REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_1,
            self.num_reward_settlements as usize,
            ErrorCode::RewardExceededMaxRewardPoolsException,
        );

        let settlement = &mut self.reward_settlements_1[self.num_reward_settlements as usize];
        self.num_reward_settlements += 1;

        Ok(settlement)
    }

    /// How to integrate multiple fields into a single array slice or whatever...
    /// You may change the return type if needed
    fn reward_settlements_mut(&mut self) -> &mut [RewardSettlement] {
        &mut self.reward_settlements_1[..self.num_reward_settlements as usize]
    }

    pub fn reward_settlements_iter_mut(&mut self) -> impl Iterator<Item = &mut RewardSettlement> {
        self.reward_settlements_mut().iter_mut()
    }

    pub fn reward_settlement_mut(&mut self, reward_id: u16) -> Option<&mut RewardSettlement> {
        self.reward_settlements_iter_mut()
            .find(|s| s.reward_id() == reward_id)
    }

    fn settle_reward(&mut self, reward_id: u16, amount: u64, current_slot: u64) -> Result<()> {
        if self.is_closed() {
            err!(ErrorCode::RewardPoolClosedError)?;
        }

        // First update contribution
        self.update_contribution(current_slot)?;

        // Find settlement and settle
        let current_reward_pool_contribution = self.contribution();
        let settlement = if let Some(settlement) = self.reward_settlement_mut(reward_id) {
            settlement
        } else {
            let reward_pool_id = self.id();
            let reward_pool_initial_slot = self.initial_slot();
            let settlement = self.allocate_new_reward_settlement()?;
            settlement.initialize(
                reward_id,
                reward_pool_id,
                reward_pool_initial_slot,
                current_slot,
            );
            settlement
        };

        settlement.settle_reward(amount, current_reward_pool_contribution, current_slot)
    }

    /// Updates the contribution of the pool into recent value.
    pub(super) fn update_contribution(&mut self, updated_slot: u64) -> Result<()> {
        let elapsed_slot = updated_slot
            .checked_sub(self.updated_slot())
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        let total_contribution_accrual_rate = self
            .token_allocated_amount
            .total_contribution_accrual_rate()?;
        let total_contribution = (elapsed_slot as u128)
            .checked_mul(total_contribution_accrual_rate as u128)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self.add_contribution(total_contribution, updated_slot)?;

        Ok(())
    }

    fn update(
        &mut self,
        deltas: Vec<TokenAllocatedAmountDelta>,
        current_slot: u64,
    ) -> Result<Vec<TokenAllocatedAmountDelta>> {
        // First update contribution
        let updated_slot = self.closed_slot().unwrap_or(current_slot);
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

/// Truncates null (0x0000) at the end.
fn from_utf8_trim_null(v: &[u8]) -> Result<String> {
    Ok(std::str::from_utf8(v)
        .map_err(|_| crate::errors::ErrorCode::DecodeInvalidUtf8FormatException)?
        .replace('\0', ""))
}
