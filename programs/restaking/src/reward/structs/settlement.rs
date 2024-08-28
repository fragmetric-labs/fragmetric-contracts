use anchor_lang::prelude::*;

use crate::error::ErrorCode;

pub const REWARD_SETTLEMENT_BLOCK_MAX_SIZE: usize = 100;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RewardSettlement {
    pub reward_id: u8,
    pub reward_pool_id: u8,

    /// Leftovers from each settlement block when clearing
    pub remaining_amount: u64,
    pub claimed_amount: u64,
    pub claimed_amount_updated_slot: u64,

    pub settled_amount: u64,
    pub settlement_blocks_last_reward_pool_contribution: u128,
    pub settlement_blocks_last_slot: u64,
    #[max_len(REWARD_SETTLEMENT_BLOCK_MAX_SIZE)]
    pub settlement_blocks: Vec<RewardSettlementBlock>,
}

impl RewardSettlement {
    pub fn new(
        reward_id: u8,
        reward_pool_id: u8,
        reward_pool_initial_slot: u64,
        current_slot: u64,
    ) -> Self {
        Self {
            reward_id,
            reward_pool_id,
            settlement_blocks: vec![],
            settlement_blocks_last_slot: reward_pool_initial_slot,
            settlement_blocks_last_reward_pool_contribution: 0,
            settled_amount: 0,
            remaining_amount: 0,
            claimed_amount: 0,
            claimed_amount_updated_slot: current_slot,
        }
    }
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UserRewardSettlement {
    pub reward_id: u8,
    pub settled_amount: u64,
    pub settled_contribution: u128,
    pub settled_slot: u64,
    pub claimed_amount: u64,
}

impl UserRewardSettlement {
    pub fn new(reward_id: u8, reward_pool_initial_slot: u64) -> Self {
        Self {
            reward_id,
            settled_amount: 0,
            claimed_amount: 0,
            settled_contribution: 0,
            settled_slot: reward_pool_initial_slot,
        }
    }
}

/// Exact settlement block range: [`starting_slot`, `ending_slot`)
#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RewardSettlementBlock {
    pub amount: u64,
    pub starting_reward_pool_contribution: u128,
    pub starting_slot: u64,
    pub ending_reward_pool_contribution: u128,
    pub ending_slot: u64,
    pub user_settled_amount: u64,
    pub user_settled_contribution: u128,
}

impl RewardSettlementBlock {
    pub fn new(
        amount: u64,
        starting_reward_pool_contribution: u128,
        starting_slot: u64,
        ending_reward_pool_contribution: u128,
        ending_slot: u64,
    ) -> Self {
        Self {
            amount,
            starting_slot,
            starting_reward_pool_contribution,
            ending_slot,
            ending_reward_pool_contribution,
            user_settled_amount: 0,
            user_settled_contribution: 0,
        }
    }

    #[inline(always)]
    pub fn block_height(&self) -> u64 {
        // SAFE: slot always monotonically increase
        self.ending_slot - self.starting_slot
    }

    #[inline(always)]
    pub fn block_contribution(&self) -> u128 {
        // SAFE: contribution always monotonically increase
        self.ending_reward_pool_contribution - self.starting_reward_pool_contribution
    }

    pub fn is_stale(&self) -> bool {
        self.user_settled_contribution == self.block_contribution()
    }

    pub fn remaining_amount(&self) -> Result<u64> {
        self.amount
            .checked_sub(self.user_settled_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))
    }
}
