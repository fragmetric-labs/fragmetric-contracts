use anchor_lang::prelude::*;
use bytemuck::Zeroable;

use crate::errors::ErrorCode;

use super::*;

const USER_REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_1: usize = 16;
// const USER_REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_2: usize = 8;

#[zero_copy]
#[repr(C, packed(8))]
pub(super) struct UserRewardPool {
    pub token_allocated_amount: TokenAllocatedAmount,
    /// user contribution at `updated_slot`
    pub contribution: u128,
    pub updated_slot: u64,
    pub reward_pool_id: u8,
    num_reward_settlements: u8,
    _padding: [u8; 6],

    _reserved: [u8; 64],

    reward_settlements_1: [UserRewardSettlement; USER_REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_1],
}

// When you want to extend user reward settlements at update v4...
// ```
// pub struct UserRewardPoolExtV4 {
//     reward_pool_id: u8,
//     num_reward_settlements: u8,
//     _padding: [u8; 14],
//     reward_settlements_2: [UserRewardSettlement; USER_REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_2],
// }
// ```
// And add new field user_reward_pools_1_ext_v4: [UserRewardPoolExtV4; USER_REWARD_ACCOUNT_REWARD_POOLS_MAX_LEN_1] to user reward account.

impl UserRewardPool {
    pub fn initialize(&mut self, reward_pool_id: u8, reward_pool_initial_slot: u64) {
        *self = Zeroable::zeroed();

        self.reward_pool_id = reward_pool_id;
        self.updated_slot = reward_pool_initial_slot;
    }

    #[inline(always)]
    pub fn get_reward_settlements_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut UserRewardSettlement> {
        self.reward_settlements_1[..self.num_reward_settlements as usize].iter_mut()
    }

    pub fn get_reward_settlement_mut(
        &mut self,
        reward_id: u16,
    ) -> Option<&mut UserRewardSettlement> {
        self.get_reward_settlements_iter_mut()
            .find(|s| s.reward_id == reward_id)
    }

    fn add_reward_settlement(
        &mut self,
        reward_id: u16,
        reward_pool_initial_slot: u64,
    ) -> Result<&mut UserRewardSettlement> {
        require_gt!(
            USER_REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_1,
            self.num_reward_settlements as usize,
            ErrorCode::RewardExceededMaxRewardSettlementError,
        );

        let settlement = &mut self.reward_settlements_1[self.num_reward_settlements as usize];
        settlement.initialize(reward_id, reward_pool_initial_slot);
        self.num_reward_settlements += 1;

        Ok(settlement)
    }

    pub fn update_reward_settlements(
        &mut self,
        reward_pool: &mut RewardPool,
        current_slot: u64,
    ) -> Result<()> {
        require_eq!(reward_pool.id, self.reward_pool_id);

        let total_contribution_accrual_rate = self
            .token_allocated_amount
            .get_total_contribution_accrual_rate();

        // Settle user reward
        let last_contribution = self.contribution;
        let last_updated_slot = self.updated_slot;
        let reward_pool_initial_slot = reward_pool.initial_slot;
        for reward_settlement in reward_pool.get_reward_settlements_iter_mut() {
            if let Some(user_reward_settlement) =
                self.get_reward_settlement_mut(reward_settlement.reward_id)
            {
                user_reward_settlement
            } else {
                self.add_reward_settlement(reward_settlement.reward_id, reward_pool_initial_slot)?
            }
            .settle_reward(
                reward_settlement,
                total_contribution_accrual_rate,
                last_contribution,
                last_updated_slot,
            )?;
        }

        // Update contribution
        let elapsed_slot = current_slot - self.updated_slot;
        self.contribution += elapsed_slot as u128 * total_contribution_accrual_rate as u128;
        self.updated_slot = current_slot;

        Ok(())
    }

    pub fn update_token_allocated_amount(
        &mut self,
        reward_pool: &mut RewardPool,
        deltas: Vec<TokenAllocatedAmountDelta>,
        current_slot: u64,
    ) -> Result<Vec<TokenAllocatedAmountDelta>> {
        // First update reward settlements
        self.update_reward_settlements(reward_pool, current_slot)?;

        // Apply deltas
        if !deltas.is_empty() {
            self.token_allocated_amount.update(deltas)
        } else {
            Ok(deltas)
        }
    }
}
