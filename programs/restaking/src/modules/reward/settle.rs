use anchor_lang::prelude::*;

use crate::errors::ErrorCode;

use super::*;

impl RewardAccount {
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
}

impl RewardPool {
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
}

impl RewardSettlement {
    fn settle_reward(
        &mut self,
        amount: u64,
        current_reward_pool_contribution: u128,
        current_slot: u64,
    ) -> Result<()> {
        let starting_reward_pool_contribution =
            self.settlement_blocks_last_reward_pool_contribution();
        let starting_slot = self.settlement_blocks_last_slot();
        let block = self.allocate_new_settlement_block()?;
        block.initialize(
            starting_reward_pool_contribution,
            starting_slot,
            current_reward_pool_contribution,
            current_slot,
        )?;
        block.add_amount(amount)?;

        self.set_settlement_blocks_last_slot(current_slot);
        self.set_settlement_blocks_last_reward_pool_contribution(current_reward_pool_contribution);
        self.add_settled_amount(amount)?;

        Ok(())
    }

    pub(super) fn clear_stale_settlement_blocks(&mut self) -> Result<usize> {
        let mut num_cleared = 0;

        while let Some(block) = self.first_settlement_block() {
            if block.is_stale() {
                let remaining_amount = block.remaining_amount()?;
                self.pop_settlement_block();
                num_cleared += 1;
                self.add_remaining_amount(remaining_amount)?;
            } else {
                break;
            }
        }

        Ok(num_cleared)
    }
}

impl UserRewardSettlement {
    pub(super) fn settle_reward(
        &mut self,
        amount: u64,
        contribution: u128,
        settled_slot: u64,
    ) -> Result<()> {
        self.add_settled_amount(amount)?;
        self.add_settled_contribution(contribution)?;
        self.update_settled_slot(settled_slot);

        Ok(())
    }
}

impl RewardSettlementBlock {
    pub(super) fn settle_user_reward(&mut self, amount: u64, contribution: u128) -> Result<()> {
        self.add_user_settled_amount(amount)?;
        self.add_user_settled_contribution(contribution)?;

        Ok(())
    }
}
