use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::errors::ErrorCode;
use crate::modules::common::PDASignerSeeds;

use super::*;

#[constant]
/// ## Version History
/// * v34: Initial Version (Data Size = 342064 ~= 335KB)
pub const REWARD_ACCOUNT_CURRENT_VERSION: u16 = 66;
const REWARD_ACCOUNT_HOLDERS_MAX_LEN_1: usize = 4;
const REWARD_ACCOUNT_REWARDS_MAX_LEN_1: usize = 16;
const REWARD_ACCOUNT_REWARD_POOLS_MAX_LEN_1: usize = 4;

#[account(zero_copy)]
#[repr(C)]
pub struct RewardAccount {
    data_version: u16,
    pub bump: u8,
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

impl PDASignerSeeds<3> for RewardAccount {
    const SEED: &'static [u8] = b"reward";

    fn signer_seeds(&self) -> [&[u8]; 3] {
        [
            Self::SEED,
            self.receipt_token_mint.as_ref(),
            self.bump_as_slice(),
        ]
    }

    fn bump_ref(&self) -> &u8 {
        &self.bump
    }
}

impl RewardAccount {
    pub fn update_if_needed(&mut self, bump: u8, receipt_token_mint: Pubkey) {
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

    /// Auxillary method to breaks down the ownership over its fields
    fn array_refs_mut(&mut self) -> (&mut [Holder], &mut [Reward], &mut [RewardPool]) {
        let holders_mut = &mut self.holders_1[..self.num_holders as usize];
        let rewards_mut = &mut self.rewards_1[..self.num_rewards as usize];
        let reward_pools_mut = &mut self.reward_pools_1[..self.num_reward_pools as usize];
        (holders_mut, rewards_mut, reward_pools_mut)
    }

    /// Auxillary method to break down ownership over its fields
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

    pub fn name(&self) -> Result<String> {
        crate::utils::from_utf8_trim_null(&self.name)
    }

    pub fn custom_contribution_accrual_rate_enabled(&self) -> bool {
        self.reward_pool_bitmap & Self::CUSTOM_CONTRIBUTION_ACCRUAL_RATE_ENABLED_BIT > 0
    }

    pub fn contribution(&self) -> u128 {
        self.contribution
    }

    pub fn add_contribution(&mut self, contribution: u128, current_slot: u64) -> Result<()> {
        self.contribution = self
            .contribution
            .checked_add(contribution)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self.updated_slot = current_slot;

        Ok(())
    }

    pub fn initial_slot(&self) -> u64 {
        self.initial_slot
    }

    pub fn updated_slot(&self) -> u64 {
        self.updated_slot
    }

    pub fn is_closed(&self) -> bool {
        self.reward_pool_bitmap & Self::IS_CLOSED_BIT > 0
    }

    pub fn set_closed(&mut self, current_slot: u64) {
        self.reward_pool_bitmap |= Self::IS_CLOSED_BIT;
        self.closed_slot = current_slot;
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
}
