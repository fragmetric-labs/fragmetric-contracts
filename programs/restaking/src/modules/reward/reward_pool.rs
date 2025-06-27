use anchor_lang::prelude::*;
use bytemuck::Zeroable;

use crate::errors::ErrorCode;

use super::*;

pub(super) const BASE_REWARD_POOL_ID: u8 = 0;
pub(super) const BONUS_REWARD_POOL_ID: u8 = 1;
const REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_1: usize = 16;
// const REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_2: usize = 8;

#[zero_copy]
#[repr(C, packed(8))]
pub(super) struct RewardPool {
    /// ID is determined by reward account.
    pub id: u8,
    /// previous field:
    /// name: [u8; 14],
    _padding: [u8; 14],

    pub custom_contribution_accrual_rate_enabled: u8,

    pub token_allocated_amount: TokenAllocatedAmount,
    pub contribution: u128,

    pub initial_slot: u64,
    pub updated_slot: u64,

    _padding2: [u8; 9],
    num_reward_settlements: u8,

    _reserved: [u8; 262],

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
    pub fn initialize(
        &mut self,
        id: u8,
        custom_contribution_accrual_rate_enabled: bool,
        current_slot: u64,
    ) -> Result<()> {
        *self = Zeroable::zeroed();

        self.id = id;
        self.custom_contribution_accrual_rate_enabled =
            custom_contribution_accrual_rate_enabled as u8;
        self.initial_slot = current_slot;
        self.updated_slot = current_slot;

        Ok(())
    }

    pub fn get_reward_settlements_iter(&self) -> impl Iterator<Item = &RewardSettlement> {
        self.reward_settlements_1[..self.num_reward_settlements as usize].iter()
    }

    pub fn get_reward_settlements_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut RewardSettlement> {
        self.reward_settlements_1[..self.num_reward_settlements as usize].iter_mut()
    }

    pub fn get_reward_settlement(&self, reward_id: u16) -> Result<&RewardSettlement> {
        self.get_reward_settlements_iter()
            .find(|s| s.reward_id == reward_id)
            .ok_or_else(|| error!(ErrorCode::RewardSettlementNotFoundError))
    }

    pub fn get_reward_settlement_mut(&mut self, reward_id: u16) -> Result<&mut RewardSettlement> {
        self.get_reward_settlements_iter_mut()
            .find(|s| s.reward_id == reward_id)
            .ok_or_else(|| error!(ErrorCode::RewardSettlementNotFoundError))
    }

    fn get_or_add_reward_settlement_mut(
        &mut self,
        reward_id: u16,
        current_slot: u64,
    ) -> Result<&mut RewardSettlement> {
        let index = self
            .get_reward_settlements_iter()
            .enumerate()
            .find_map(|(i, settlement)| (settlement.reward_id == reward_id).then_some(i));

        let index = match index {
            Some(index) => index,
            None => {
                require_gt!(
                    REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_1,
                    self.num_reward_settlements as usize,
                    ErrorCode::RewardExceededMaxRewardPoolsError,
                );

                self.reward_settlements_1[self.num_reward_settlements as usize].initialize(
                    reward_id,
                    self.id,
                    self.initial_slot,
                    current_slot,
                );
                self.num_reward_settlements += 1;
                self.num_reward_settlements as usize - 1
            }
        };

        Ok(&mut self.reward_settlements_1[index])
    }

    pub fn get_unclaimed_reward_amount(&self, reward_id: u16) -> u64 {
        self.get_reward_settlement(reward_id)
            .map(|settlement| settlement.get_unclaimed_reward_amount())
            .unwrap_or_default()
    }

    /// Updates the contribution of the pool into recent value.
    ///
    /// this operation is idempotent
    fn update_contribution(&mut self, current_slot: u64) {
        if current_slot <= self.updated_slot {
            return;
        }

        let total_contribution_accrual_rate = self
            .token_allocated_amount
            .get_total_contribution_accrual_rate();
        self.contribution +=
            (current_slot - self.updated_slot) as u128 * total_contribution_accrual_rate;
        self.updated_slot = current_slot;
    }

    /// Updates the contribution of the pool and clear stale settlement blocks.
    ///
    /// this operation is idempotent
    pub fn update_reward_pool(&mut self, current_slot: u64) {
        // First update contribution
        self.update_contribution(current_slot);

        // Clear stale blocks
        self.get_reward_settlements_iter_mut()
            .for_each(|settlement| settlement.clear_stale_settlement_blocks());
    }

    /// add new settlement block to corresponding reward settlement
    pub fn settle_reward(&mut self, reward_id: u16, amount: u64, current_slot: u64) -> Result<()> {
        // First update contribution
        self.update_contribution(current_slot);

        // Find settlement and settle
        let current_reward_pool_contribution = self.contribution;
        self.get_or_add_reward_settlement_mut(reward_id, current_slot)?
            .settle_reward(amount, current_reward_pool_contribution, current_slot)
    }

    /// Updates the token allocated amount and contribution of the pool into recent value.
    pub fn update_token_allocated_amount(
        &mut self,
        deltas: Vec<TokenAllocatedAmountDelta>,
        current_slot: u64,
    ) -> Result<Vec<TokenAllocatedAmountDelta>> {
        // First update contribution
        self.update_contribution(current_slot);

        // Apply deltas
        if !deltas.is_empty() {
            self.token_allocated_amount.update(deltas)
        } else {
            Ok(deltas)
        }
    }

    /// returns claimed_amount
    pub fn claim_remaining_reward(&mut self, reward_id: u16, current_slot: u64) -> Result<u64> {
        let Ok(settlement) = self.get_reward_settlement_mut(reward_id) else {
            return Ok(0);
        };

        settlement.claim_remaining_reward(current_slot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_token_allocated_amount() {
        let mut pool = RewardPool::zeroed();

        let mut current_slot = 10;
        pool.update_token_allocated_amount(
            vec![
                TokenAllocatedAmountDelta::new_positive(None, 50),
                TokenAllocatedAmountDelta::new_positive(Some(130), 100),
            ],
            current_slot,
        )
        .unwrap();

        let contribution = pool.contribution;
        assert_eq!(contribution, 0);
        assert_eq!(
            pool.token_allocated_amount
                .get_total_contribution_accrual_rate(),
            180_00,
        );
        assert_eq!(pool.updated_slot, current_slot);

        current_slot = 20;
        pool.update_token_allocated_amount(
            vec![TokenAllocatedAmountDelta::new_negative(100)],
            current_slot,
        )
        .unwrap();

        let contribution = pool.contribution;
        assert_eq!(contribution, 180_000);
        assert_eq!(
            pool.token_allocated_amount
                .get_total_contribution_accrual_rate(),
            65_00,
        );
        assert_eq!(pool.updated_slot, current_slot);
    }
}
