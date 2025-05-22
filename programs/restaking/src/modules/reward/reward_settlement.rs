use anchor_lang::prelude::*;
use bytemuck::Zeroable;
use primitive_types::U256;

use crate::errors::ErrorCode;

const REWARD_ACCOUNT_SETTLEMENT_BLOCK_MAX_LEN: usize = 64;

#[zero_copy]
#[repr(C, packed(8))]
pub(super) struct RewardSettlement {
    pub reward_id: u16,
    pub reward_pool_id: u8,
    num_settlement_blocks: u8,
    settlement_blocks_head: u8,
    settlement_blocks_tail: u8,
    _padding2: [u8; 2],

    /// Leftovers from each settlement block when clearing
    pub remaining_amount: u64,
    pub claimed_amount: u64,
    pub claimed_amount_updated_slot: u64,

    // First block starts from the beginning of reward pool
    pub settled_amount: u64,
    pub settlement_blocks_last_slot: u64,
    pub settlement_blocks_last_reward_pool_contribution: u128,
    settlement_blocks: [RewardSettlementBlock; REWARD_ACCOUNT_SETTLEMENT_BLOCK_MAX_LEN],
}

impl RewardSettlement {
    pub fn initialize(
        &mut self,
        reward_id: u16,
        reward_pool_id: u8,
        reward_pool_initial_slot: u64,
        current_slot: u64,
    ) {
        *self = Zeroable::zeroed();

        self.reward_id = reward_id;
        self.reward_pool_id = reward_pool_id;
        self.claimed_amount_updated_slot = current_slot;
        self.settlement_blocks_last_slot = reward_pool_initial_slot;
    }

    pub fn get_settlement_blocks_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut RewardSettlementBlock> {
        let (front, back) = self
            .settlement_blocks
            .split_at_mut(self.settlement_blocks_head as usize);
        let back_len = self
            .num_settlement_blocks
            .min(REWARD_ACCOUNT_SETTLEMENT_BLOCK_MAX_LEN as u8 - self.settlement_blocks_head)
            as usize;
        let front_len = self.num_settlement_blocks as usize - back_len;
        back.iter_mut()
            .take(back_len)
            .chain(front.iter_mut().take(front_len))
    }

    /// this operation is idempotent
    pub fn clear_stale_settlement_blocks(&mut self) {
        for _ in 0..self.num_settlement_blocks {
            let block = &mut self.settlement_blocks[self.settlement_blocks_head as usize];
            if block.is_stale() {
                self.remaining_amount += block.get_remaining_amount();
                // pop_front
                self.settlement_blocks_head = (self.settlement_blocks_head + 1)
                    % REWARD_ACCOUNT_SETTLEMENT_BLOCK_MAX_LEN as u8;
                self.num_settlement_blocks -= 1;
            } else {
                return;
            }
        }
    }

    /// first clear stale settlement blocks and then create new settlement block.
    pub fn settle_reward(
        &mut self,
        amount: u64,
        current_reward_pool_contribution: u128,
        current_slot: u64,
    ) -> Result<()> {
        self.clear_stale_settlement_blocks();
        self.add_settlement_block(amount, current_reward_pool_contribution, current_slot)?;

        self.settled_amount += amount;
        self.settlement_blocks_last_slot = current_slot;
        self.settlement_blocks_last_reward_pool_contribution = current_reward_pool_contribution;

        Ok(())
    }

    fn add_settlement_block(
        &mut self,
        amount: u64,
        current_reward_pool_contribution: u128,
        current_slot: u64,
    ) -> Result<()> {
        // Prevent settlement block with non-positive block height
        require_gt!(
            current_slot,
            self.settlement_blocks_last_slot,
            ErrorCode::RewardInvalidSettlementBlockHeightException,
        );

        // Prevent settlement block with negative block contribution
        require_gte!(
            current_reward_pool_contribution,
            self.settlement_blocks_last_reward_pool_contribution,
            ErrorCode::RewardInvalidSettlementBlockContributionException,
        );

        if self.num_settlement_blocks as usize >= REWARD_ACCOUNT_SETTLEMENT_BLOCK_MAX_LEN {
            self.force_clear_settlement_block();
        }

        // push_back
        self.settlement_blocks[self.settlement_blocks_tail as usize].initialize(
            amount,
            self.settlement_blocks_last_reward_pool_contribution,
            self.settlement_blocks_last_slot,
            current_reward_pool_contribution,
            current_slot,
        );
        self.settlement_blocks_tail =
            (self.settlement_blocks_tail + 1) % REWARD_ACCOUNT_SETTLEMENT_BLOCK_MAX_LEN as u8;
        self.num_settlement_blocks += 1;

        Ok(())
    }

    fn force_clear_settlement_block(&mut self) {
        let block = &self.settlement_blocks[self.settlement_blocks_head as usize];
        self.remaining_amount += block.get_remaining_amount();
        
        self.settlement_blocks_head = (self.settlement_blocks_head + 1) % REWARD_ACCOUNT_SETTLEMENT_BLOCK_MAX_LEN as u8;
        self.num_settlement_blocks -= 1;
    }

    pub fn get_unclaimed_reward_amount(&self) -> u64 {
        self.settled_amount - self.claimed_amount
    }

    pub fn claim_user_reward(&mut self, amount: u64, current_slot: u64) -> Result<()> {
        require_gte!(current_slot, self.claimed_amount_updated_slot);
        require_gte!(self.settled_amount, self.claimed_amount + amount);

        self.claimed_amount += amount;
        self.claimed_amount_updated_slot = current_slot;

        Ok(())
    }
}

/// Exact settlement block range: [`starting_slot`, `ending_slot`)
#[zero_copy]
#[repr(C, packed(8))]
pub(super) struct RewardSettlementBlock {
    pub amount: u64,
    pub starting_slot: u64,
    pub starting_reward_pool_contribution: u128,
    pub ending_reward_pool_contribution: u128,
    pub ending_slot: u64,
    pub user_settled_amount: u64,
    pub user_settled_contribution: u128,
}

impl RewardSettlementBlock {
    fn initialize(
        &mut self,
        amount: u64,
        starting_reward_pool_contribution: u128,
        starting_slot: u64,
        ending_reward_pool_contribution: u128,
        ending_slot: u64,
    ) {
        *self = Zeroable::zeroed();

        self.amount = amount;
        self.starting_slot = starting_slot;
        self.starting_reward_pool_contribution = starting_reward_pool_contribution;
        self.ending_slot = ending_slot;
        self.ending_reward_pool_contribution = ending_reward_pool_contribution;
    }

    #[inline(always)]
    pub fn get_block_contribution(&self) -> u128 {
        self.ending_reward_pool_contribution - self.starting_reward_pool_contribution
    }

    #[inline(always)]
    pub fn is_stale(&self) -> bool {
        self.user_settled_contribution == self.get_block_contribution()
    }

    #[inline(always)]
    pub fn get_remaining_amount(&self) -> u64 {
        self.amount - self.user_settled_amount
    }

    /// returns user settled amount
    pub fn settle_user_reward(&mut self, contribution: u128) -> Result<u64> {
        if contribution == 0 {
            return Ok(0);
        }

        let amount = u64::try_from(
            U256::from(contribution)
                .checked_mul(U256::from(self.amount))
                .and_then(|x| x.checked_div(U256::from(self.get_block_contribution())))
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?,
        )
        .map_err(|_| error!(ErrorCode::CalculationArithmeticException))?;

        self.user_settled_amount += amount;
        self.user_settled_contribution += contribution;

        require_gte!(
            self.amount,
            self.user_settled_amount,
            ErrorCode::RewardInvalidTotalUserSettledAmountException
        );
        require_gte!(
            self.get_block_contribution(),
            self.user_settled_contribution,
            ErrorCode::RewardInvalidTotalUserSettledContributionException
        );

        Ok(amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settlement() {
        let mut settlement = RewardSettlement::zeroed();
        settlement.initialize(0, 0, 0, 0);
        settlement.settlement_blocks_head = 61;
        settlement.settlement_blocks_tail = 61;

        settlement.settle_reward(63, 2, 1).unwrap(); // idx = 61
        settlement.settle_reward(64, 4, 2).unwrap(); // idx = 62

        assert_eq!(settlement.settled_amount, 127);
        let settlement_blocks_last_reward_pool_contribution =
            settlement.settlement_blocks_last_reward_pool_contribution;
        assert_eq!(settlement_blocks_last_reward_pool_contribution, 4);
        assert_eq!(settlement.settlement_blocks_last_slot, 2);
        assert_eq!(settlement.num_settlement_blocks, 2);
        assert_eq!(settlement.settlement_blocks_head, 61);
        assert_eq!(settlement.settlement_blocks_tail, 63);
        assert_eq!(settlement.get_settlement_blocks_iter_mut().count(), 2);
        settlement
            .get_settlement_blocks_iter_mut()
            .enumerate()
            .for_each(|(i, block)| assert_eq!(block.amount, 2 + (61 + i as u64) % 64));

        let block = &mut settlement.settlement_blocks[61];
        let amount = block.settle_user_reward(1).unwrap();
        let user_settled_contribution = block.user_settled_contribution;
        assert_eq!(amount, 31);
        assert_eq!(user_settled_contribution, 1);
        assert_eq!(block.user_settled_amount, 31);

        let amount = block.settle_user_reward(1).unwrap();
        let user_settled_contribution = block.user_settled_contribution;
        assert_eq!(amount, 31);
        assert_eq!(user_settled_contribution, 2);
        assert_eq!(block.user_settled_amount, 62);
        assert!(block.is_stale());

        settlement.clear_stale_settlement_blocks();

        assert_eq!(settlement.remaining_amount, 1);
        assert_eq!(settlement.num_settlement_blocks, 1);
        assert_eq!(settlement.settlement_blocks_head, 62);
        assert_eq!(settlement.settlement_blocks_tail, 63);
        assert_eq!(settlement.get_settlement_blocks_iter_mut().count(), 1);
        settlement
            .get_settlement_blocks_iter_mut()
            .enumerate()
            .for_each(|(i, block)| assert_eq!(block.amount, 2 + (62 + i as u64) % 64));

        settlement.settle_reward(65, 6, 3).unwrap(); // idx = 63

        assert_eq!(settlement.settled_amount, 192);
        let settlement_blocks_last_reward_pool_contribution =
            settlement.settlement_blocks_last_reward_pool_contribution;
        assert_eq!(settlement_blocks_last_reward_pool_contribution, 6);
        assert_eq!(settlement.settlement_blocks_last_slot, 3);
        assert_eq!(settlement.num_settlement_blocks, 2);
        assert_eq!(settlement.settlement_blocks_head, 62);
        assert_eq!(settlement.settlement_blocks_tail, 0);
        assert_eq!(settlement.get_settlement_blocks_iter_mut().count(), 2);
        settlement
            .get_settlement_blocks_iter_mut()
            .enumerate()
            .for_each(|(i, block)| assert_eq!(block.amount, 2 + (62 + i as u64) % 64));

        settlement.settle_reward(2, 10, 5).unwrap(); // idx = 0

        assert_eq!(settlement.settled_amount, 194);
        let settlement_blocks_last_reward_pool_contribution =
            settlement.settlement_blocks_last_reward_pool_contribution;
        assert_eq!(settlement_blocks_last_reward_pool_contribution, 10);
        assert_eq!(settlement.settlement_blocks_last_slot, 5);
        assert_eq!(settlement.num_settlement_blocks, 3);
        assert_eq!(settlement.settlement_blocks_head, 62);
        assert_eq!(settlement.settlement_blocks_tail, 1);
        assert_eq!(settlement.get_settlement_blocks_iter_mut().count(), 3);
        settlement
            .get_settlement_blocks_iter_mut()
            .enumerate()
            .for_each(|(i, block)| assert_eq!(block.amount, 2 + (62 + i as u64) % 64));

        settlement.settlement_blocks[62]
            .settle_user_reward(2)
            .unwrap();
        settlement.settlement_blocks[63]
            .settle_user_reward(2)
            .unwrap();
        settlement.clear_stale_settlement_blocks();

        assert_eq!(settlement.remaining_amount, 1);
        assert_eq!(settlement.num_settlement_blocks, 1);
        assert_eq!(settlement.settlement_blocks_head, 0);
        assert_eq!(settlement.settlement_blocks_tail, 1);
        assert_eq!(settlement.get_settlement_blocks_iter_mut().count(), 1);
        settlement
            .get_settlement_blocks_iter_mut()
            .enumerate()
            .for_each(|(i, block)| assert_eq!(block.amount, 2 + i as u64 % 64));
    }
}
