use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::errors::ErrorCode;

const REWARD_SETTLEMENT_BLOCK_MAX_LEN: usize = 64;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct RewardSettlement {
    reward_id: u16,
    reward_pool_id: u8,
    num_settlement_blocks: u8,
    settlement_blocks_head: u8,
    settlement_blocks_tail: u8,
    _padding: [u8; 2],

    /// Leftovers from each settlement block when clearing
    remaining_amount: u64,
    claimed_amount: u64,
    claimed_amount_updated_slot: u64,

    settled_amount: u64,
    settlement_blocks_last_slot: u64,
    settlement_blocks_last_reward_pool_contribution: u128,
    settlement_blocks: [RewardSettlementBlock; REWARD_SETTLEMENT_BLOCK_MAX_LEN],
}

impl RewardSettlement {
    pub fn initialize(
        &mut self,
        reward_id: u16,
        reward_pool_id: u8,
        reward_pool_initial_slot: u64,
        current_slot: u64,
    ) {
        self.reward_id = reward_id;
        self.reward_pool_id = reward_pool_id;
        self.num_settlement_blocks = 0;
        self.settlement_blocks_head = 0;
        self.settlement_blocks_tail = 0;
        self.remaining_amount = 0;
        self.claimed_amount = 0;
        self.claimed_amount_updated_slot = current_slot;
        self.settled_amount = 0;
        self.settlement_blocks_last_slot = reward_pool_initial_slot;
        self.settlement_blocks_last_reward_pool_contribution = 0;
    }

    pub fn reward_id(&self) -> u16 {
        self.reward_id
    }

    fn is_settlement_blocks_full(&self) -> bool {
        self.num_settlement_blocks as usize == REWARD_SETTLEMENT_BLOCK_MAX_LEN
    }

    fn is_settlement_blocks_empty(&self) -> bool {
        self.num_settlement_blocks == 0
    }

    pub fn add_remaining_amount(&mut self, amount: u64) -> Result<()> {
        self.remaining_amount = self
            .remaining_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }

    pub fn add_settled_amount(&mut self, amount: u64) -> Result<()> {
        self.settled_amount = self
            .settled_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }

    pub fn settlement_blocks_last_slot(&self) -> u64 {
        self.settlement_blocks_last_slot
    }

    pub fn set_settlement_blocks_last_slot(&mut self, current_slot: u64) {
        self.settlement_blocks_last_slot = current_slot;
    }

    pub fn settlement_blocks_last_reward_pool_contribution(&self) -> u128 {
        self.settlement_blocks_last_reward_pool_contribution
    }

    pub fn set_settlement_blocks_last_reward_pool_contribution(
        &mut self,
        current_reward_pool_contribution: u128,
    ) {
        self.settlement_blocks_last_reward_pool_contribution = current_reward_pool_contribution;
    }

    pub fn allocate_new_settlement_block(&mut self) -> Result<&mut RewardSettlementBlock> {
        if self.is_settlement_blocks_full() && self.clear_stale_settlement_blocks()? == 0 {
            err!(ErrorCode::RewardStaleSettlementBlockNotExistError)?
        }

        let block = &mut self.settlement_blocks[self.settlement_blocks_tail as usize];
        self.settlement_blocks_tail =
            (self.settlement_blocks_tail + 1) % (REWARD_SETTLEMENT_BLOCK_MAX_LEN as u8);
        self.num_settlement_blocks += 1;

        Ok(block)
    }

    pub fn pop_settlement_block(&mut self) {
        if !self.is_settlement_blocks_empty() {
            self.settlement_blocks_head =
                (self.settlement_blocks_head + 1) % (REWARD_SETTLEMENT_BLOCK_MAX_LEN as u8);
            self.num_settlement_blocks -= 1;
        }
    }

    pub fn first_settlement_block(&self) -> Option<&RewardSettlementBlock> {
        (!self.is_settlement_blocks_empty())
            .then(|| &self.settlement_blocks[self.settlement_blocks_head as usize])
    }

    fn is_settlement_blocks_queue_partitioned(&self) -> bool {
        self.is_settlement_blocks_full()
            || self.settlement_blocks_head > self.settlement_blocks_tail
    }

    pub fn settlement_blocks_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut RewardSettlementBlock> {
        if self.is_settlement_blocks_queue_partitioned() {
            let (front, back) = self
                .settlement_blocks
                .split_at_mut(self.settlement_blocks_head as usize);
            // This actual type is std::iter::Chain<std::slice::IterMut<'_, _>, std::iter::Take<std::slice::IterMut<'_, RewardSettlementBlock>>>
            back.iter_mut()
                .chain(front.iter_mut().take(self.settlement_blocks_tail as usize))
        } else {
            // Chain with empty iterator to make type compatible
            // without empty iterator, its actual type is std::slice::IterMut<'_, &mut RewardSettlementBlock>
            // with empty iterator, its actual type is std::iter::Chain<std::slice::IterMut<'_, _>, std::iter::Take<std::slice::IterMut<'_, RewardSettlementBlock>>>
            self.settlement_blocks
                [self.settlement_blocks_head as usize..self.settlement_blocks_tail as usize]
                .iter_mut()
                .chain([].iter_mut().take(0))
        }
    }
}

/// Exact settlement block range: [`starting_slot`, `ending_slot`)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct RewardSettlementBlock {
    amount: u64,
    starting_slot: u64,
    starting_reward_pool_contribution: u128,
    ending_reward_pool_contribution: u128,
    ending_slot: u64,
    user_settled_amount: u64,
    user_settled_contribution: u128,
}

impl RewardSettlementBlock {
    pub fn initialize(
        &mut self,
        starting_reward_pool_contribution: u128,
        starting_slot: u64,
        ending_reward_pool_contribution: u128,
        ending_slot: u64,
    ) -> Result<()> {
        // Prevent settlement block with non-positive block height
        if starting_slot >= ending_slot {
            err!(ErrorCode::RewardInvalidSettlementBlockHeightException)?
        }

        // Prevent settlement block with negative block contribution
        if starting_reward_pool_contribution > ending_reward_pool_contribution {
            err!(ErrorCode::RewardInvalidSettlementBlockContributionException)?
        }

        self.amount = 0;
        self.starting_slot = starting_slot;
        self.starting_reward_pool_contribution = starting_reward_pool_contribution;
        self.ending_slot = ending_slot;
        self.ending_reward_pool_contribution = ending_reward_pool_contribution;
        self.user_settled_amount = 0;
        self.user_settled_contribution = 0;

        Ok(())
    }

    pub fn amount(&self) -> u64 {
        self.amount
    }

    pub fn add_amount(&mut self, amount: u64) -> Result<()> {
        self.amount = self
            .amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        Ok(())
    }

    pub fn starting_slot(&self) -> u64 {
        self.starting_slot
    }

    pub fn ending_slot(&self) -> u64 {
        self.ending_slot
    }

    pub fn add_user_settled_amount(&mut self, amount: u64) -> Result<()> {
        self.user_settled_amount = self
            .user_settled_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        if self.user_settled_amount > self.amount {
            err!(ErrorCode::RewardInvalidTotalUserSettledAmountException)?
        }

        Ok(())
    }

    pub fn add_user_settled_contribution(&mut self, contribution: u128) -> Result<()> {
        self.user_settled_contribution =
            self.user_settled_contribution
                .checked_add(contribution)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        if self.user_settled_contribution > self.block_contribution() {
            err!(ErrorCode::RewardInvalidTotalUserSettledContributionException)?
        }

        Ok(())
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

    #[inline(always)]
    pub fn is_stale(&self) -> bool {
        self.user_settled_contribution == self.block_contribution()
    }

    #[inline(always)]
    pub fn remaining_amount(&self) -> Result<u64> {
        self.amount
            .checked_sub(self.user_settled_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }
}
