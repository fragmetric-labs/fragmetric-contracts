use anchor_lang::prelude::*;

use crate::{error::ErrorCode, reward::*};

impl RewardPool {
    pub(super) fn settle_reward(
        &mut self,
        reward_id: u8,
        amount: u64,
        current_slot: u64,
    ) -> Result<()> {
        if self.closed_slot.is_some() {
            err!(ErrorCode::RewardPoolAlreadyClosed)?;
        }

        // First update contribution
        self.update_contribution(current_slot)?;

        // Find settlement and settle
        if let Some(settlement) = self
            .reward_settlements
            .iter_mut()
            .find(|s| s.reward_id == reward_id)
        {
            settlement.settle_reward(amount, self.contribution, current_slot)
        } else {
            let mut settlement =
                RewardSettlement::new(reward_id, self.id, self.initial_slot, current_slot);
            settlement.settle_reward(amount, self.contribution, current_slot)?;
            self.reward_settlements.push(settlement);
            Ok(())
        }
    }
}

impl RewardSettlement {
    fn settle_reward(
        &mut self,
        amount: u64,
        current_reward_pool_contribution: u128,
        current_slot: u64,
    ) -> Result<()> {
        if self.settlement_blocks.len() == Self::REWARD_SETTLEMENT_BLOCK_MAX_SIZE
            && self.clear_stale_settlement_blocks()? == 0
        {
            err!(ErrorCode::RewardStaleSettlementBlockDoesNotExist)?;
        }

        let starting_slot = self.settlement_blocks_last_slot;
        let ending_slot = current_slot;

        // Prevent settlement block with non-positive block height
        if starting_slot >= ending_slot {
            err!(ErrorCode::RewardInvalidSettlementBlockHeight)?;
        }

        let starting_reward_pool_contribution =
            self.settlement_blocks_last_reward_pool_contribution;
        let ending_reward_pool_contribution = current_reward_pool_contribution;

        // Prevent settlement block with negative block contribution
        if starting_reward_pool_contribution > ending_reward_pool_contribution {
            err!(ErrorCode::RewardInvalidSettlementBlockContribution)?;
        }

        let settlement_block = RewardSettlementBlock::new(
            amount,
            starting_reward_pool_contribution,
            starting_slot,
            ending_reward_pool_contribution,
            ending_slot,
        );
        self.settlement_blocks.push(settlement_block);
        self.settlement_blocks_last_slot = current_slot;
        self.settlement_blocks_last_reward_pool_contribution = current_reward_pool_contribution;
        self.settled_amount = self
            .settled_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;

        Ok(())
    }

    pub(super) fn clear_stale_settlement_blocks(&mut self) -> Result<usize> {
        let mut cleared = 0;
        let mut iter = std::mem::take(&mut self.settlement_blocks)
            .into_iter()
            .peekable();
        while let Some(block) = iter.peek() {
            if block.is_stale() {
                // first()
                let block = iter.next().unwrap(); // pop()
                cleared += 1;
                self.remaining_amount = self
                    .remaining_amount
                    .checked_add(block.remaining_amount()?)
                    .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
            } else {
                break;
            }
        }
        self.settlement_blocks = iter.collect();

        Ok(cleared)
    }
}

impl UserRewardSettlement {
    pub(super) fn settle_reward(
        &mut self,
        amount: u64,
        contribution: u128,
        settled_slot: u64,
    ) -> Result<()> {
        self.settled_amount = self
            .settled_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
        self.settled_contribution = self
            .settled_contribution
            .checked_add(contribution)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
        self.settled_slot = settled_slot;

        Ok(())
    }
}

impl RewardSettlementBlock {
    pub(super) fn settle_user_reward(&mut self, amount: u64, contribution: u128) -> Result<()> {
        self.user_settled_amount = self
            .user_settled_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
        self.user_settled_contribution = self
            .user_settled_contribution
            .checked_add(contribution)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;

        if self.user_settled_amount > self.amount {
            err!(ErrorCode::RewardInvalidTotalUserSettledAmount)?;
        }

        if self.user_settled_contribution > self.block_contribution() {
            err!(ErrorCode::RewardInvalidTotalUserSettledContribution)?;
        }

        Ok(())
    }
}
