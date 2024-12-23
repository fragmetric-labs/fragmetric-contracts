use anchor_lang::prelude::*;
use bytemuck::Zeroable;

use crate::errors::ErrorCode;

use super::*;

const REWARD_POOL_NAME_MAX_LEN: usize = 14;
const REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_1: usize = 16;
// const REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_2: usize = 8;

#[zero_copy]
#[repr(C)]
pub struct RewardPool {
    /// ID is determined by reward account.
    pub(super) id: u8,
    name: [u8; REWARD_POOL_NAME_MAX_LEN],

    // bit 0: custom contribution accrual rate enabled?
    // bit 1: is closed?
    // bit 2: has holder? (not provided for default holder (fragmetric))
    reward_pool_bitmap: u8,

    token_allocated_amount: TokenAllocatedAmount,
    contribution: u128,

    pub(super) initial_slot: u64,
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

    pub(super) fn initialize(
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

    pub(super) fn get_name(&self) -> Result<&str> {
        Ok(std::str::from_utf8(&self.name)
            .map_err(|_| crate::errors::ErrorCode::UTF8DecodingException)?
            .trim_matches('\0'))
    }

    #[inline(always)]
    pub(super) fn is_custom_contribution_accrual_rate_enabled(&self) -> bool {
        self.reward_pool_bitmap & Self::CUSTOM_CONTRIBUTION_ACCRUAL_RATE_ENABLED_BIT > 0
    }

    #[inline(always)]
    pub(super) fn get_closed_slot(&self) -> Option<u64> {
        self.is_closed().then_some(self.closed_slot)
    }

    #[inline(always)]
    fn is_closed(&self) -> bool {
        self.reward_pool_bitmap & Self::IS_CLOSED_BIT > 0
    }

    #[inline(always)]
    fn set_closed(&mut self, closed_slot: u64) {
        self.reward_pool_bitmap |= Self::IS_CLOSED_BIT;
        self.closed_slot = closed_slot;
    }

    #[inline(always)]
    pub(super) fn get_holder_id(&self) -> Option<u8> {
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
            ErrorCode::RewardExceededMaxRewardPoolsError,
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
    ) -> impl Iterator<Item = &mut RewardSettlement> {
        self.get_reward_settlements_mut().iter_mut()
    }

    fn get_reward_settlement_mut(&mut self, reward_id: u16) -> Option<&mut RewardSettlement> {
        self.get_reward_settlements_iter_mut()
            .find(|s| s.reward_id == reward_id)
    }

    pub(super) fn settle_reward(
        &mut self,
        reward_id: u16,
        amount: u64,
        current_slot: u64,
    ) -> Result<()> {
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
    pub(super) fn update_contribution(&mut self, updated_slot: u64) -> Result<()> {
        let elapsed_slot = updated_slot
            .checked_sub(self.updated_slot)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        if elapsed_slot == 0 {
            return Ok(());
        }

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

    /// Updates the token allocated amount and contribution of the pool into recent value.
    pub(super) fn update(
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

    pub(super) fn close(&mut self, current_slot: u64) -> Result<()> {
        if self.is_closed() {
            err!(ErrorCode::RewardPoolClosedError)?
        }

        // update contribution as last
        self.update_contribution(current_slot)?;
        self.set_closed(current_slot);

        Ok(())
    }
}
