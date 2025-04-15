use std::ops::Mul;
use anchor_lang::prelude::*;
use bytemuck::Zeroable;
use primitive_types::U256;

use crate::errors::ErrorCode;

use super::*;

const USER_REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_1: usize = 16;
// const USER_REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_2: usize = 8;

#[zero_copy]
#[repr(C)]
pub struct UserRewardPool {
    token_allocated_amount: TokenAllocatedAmount,
    contribution: u128,
    updated_slot: u64,
    reward_pool_id: u8,
    num_reward_settlements: u8,
    _padding: [u8; 6],

    _reserved: [u64; 8],

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
    pub(super) fn initialize(&mut self, reward_pool_id: u8, reward_pool_initial_slot: u64) {
        self.token_allocated_amount = TokenAllocatedAmount::zeroed();
        self.contribution = 0;
        self.updated_slot = reward_pool_initial_slot;
        self.reward_pool_id = reward_pool_id;
        self.num_reward_settlements = 0;
    }

    fn add_new_reward_settlement(
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

    /// How to integrate multiple fields into a single array slice or whatever...
    /// You may change the return type if needed
    #[inline(always)]
    fn get_reward_settlements_mut(&mut self) -> &mut [UserRewardSettlement] {
        &mut self.reward_settlements_1[..self.num_reward_settlements as usize]
    }

    #[inline(always)]
    fn get_reward_settlements_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut UserRewardSettlement> {
        self.get_reward_settlements_mut().iter_mut()
    }

    fn get_reward_settlement_mut(&mut self, reward_id: u16) -> Option<&mut UserRewardSettlement> {
        self.get_reward_settlements_iter_mut()
            .find(|s| s.reward_id == reward_id)
    }

    pub(super) fn update(
        &mut self,
        reward_pool: &mut RewardPool,
        deltas: Vec<TokenAllocatedAmountDelta>,
        current_slot: u64,
    ) -> Result<Vec<TokenAllocatedAmountDelta>> {
        // cache value
        let total_contribution_accrual_rate = self
            .token_allocated_amount
            .get_total_contribution_accrual_rate()?;

        // First update contribution, but save old data for settlement
        let last_contribution = self.contribution;
        let last_updated_slot = self.updated_slot;
        let updated_slot = reward_pool.get_closed_slot().unwrap_or(current_slot);
        self.update_contribution(updated_slot, total_contribution_accrual_rate)?;

        // Settle user reward
        let reward_pool_initial_slot = reward_pool.initial_slot;
        for reward_settlement in reward_pool.get_reward_settlements_iter_mut() {
            // Find corresponding user reward settlement
            let user_reward_settlement = if let Some(user_reward_settlement) =
                self.get_reward_settlement_mut(reward_settlement.reward_id)
            {
                user_reward_settlement
            } else {
                self.add_new_reward_settlement(
                    reward_settlement.reward_id,
                    reward_pool_initial_slot,
                )?
            };

            for block in reward_settlement.get_settlement_blocks_iter_mut() {
                let user_block_settled_contribution = if last_updated_slot < block.starting_slot {
                    // case 1: ...updated...[starting...ending)...
                    (block.get_block_height() as u128)
                        .checked_mul(total_contribution_accrual_rate as u128)
                        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?
                } else if last_updated_slot <= block.ending_slot {
                    // case 2: ...[starting...updated...ending)...
                    //
                    // Special case: updated == ending
                    //
                    // In this case this settlement block has been settled at the same slot
                    // when user reward pool has been updated.
                    // Therefore we have to check settled_slot == ending_slot to determine
                    // if this block is already settled. However, it could be ignored
                    // since the calculation logic below will return 0.
                    let first_half =
                        last_contribution - user_reward_settlement.settled_contribution; // SAFE: contribution always monotonically increase
                    let second_half = ((block.ending_slot - last_updated_slot) as u128)
                        .checked_mul(total_contribution_accrual_rate as u128)
                        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
                    first_half
                        .checked_add(second_half)
                        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?
                } else {
                    // case 3: [starting...ending)...updated...
                    //
                    // This block has already been handled so skip
                    continue;
                };

                // If block contribution is zero, then user contribution is also zero.
                // Why? If block height = 0 then obvious.
                // If total allocated amount is zero then user's allocated amount is also zero.
                // Therefore nobody can claim for this settlement block, and the block is stale.
                let block_contribution = block.get_block_contribution();
                let user_block_settled_amount = (block_contribution > 0)
                    .then(|| {
                        u64::try_from(
                            U256::from(user_block_settled_contribution)
                                .checked_mul(U256::from(block.amount))
                                .and_then(|x| x.checked_div(U256::from(block_contribution)))
                                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?
                        )
                            .map_err(|_| error!(ErrorCode::CalculationArithmeticException))
                    })
                    .transpose()?
                    .unwrap_or_default();
                // // is equivalent to:
                // let user_block_settled_amount = if block_contribution > 0 {
                //     u64::try_from(
                //         user_block_settled_contribution
                //             .checked_mul(block.amount as u128)
                //             .and_then(|x| x.checked_div(block_contribution))
                //             .ok_or_else(|| error!(ErrorCode::CalculationFailure))?,
                //     )
                //     .map_err(|_| error!(ErrorCode::CalculationFailure))?
                // } else {
                //     0
                // };

                user_reward_settlement.settle_reward(
                    user_block_settled_amount,
                    user_block_settled_contribution,
                    block.ending_slot,
                )?;

                // to find out stale blocks;
                block.settle_user_reward(
                    user_block_settled_amount,
                    user_block_settled_contribution,
                )?;
            }
        }

        // Apply deltas
        if !deltas.is_empty() {
            self.token_allocated_amount.update(deltas)
        } else {
            Ok(deltas)
        }
    }

    fn update_contribution(
        &mut self,
        updated_slot: u64,
        total_contribution_accrual_rate: u64, // cached
    ) -> Result<()> {
        let elapsed_slot = updated_slot
            .checked_sub(self.updated_slot)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        if elapsed_slot == 0 {
            return Ok(());
        }

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
}
