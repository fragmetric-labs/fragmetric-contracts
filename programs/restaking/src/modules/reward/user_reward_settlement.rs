use anchor_lang::prelude::*;
use bytemuck::Zeroable;

use crate::errors::ErrorCode;

use super::*;

#[zero_copy]
#[repr(C, packed(8))]
pub(super) struct UserRewardSettlement {
    pub reward_id: u16,
    _padding: [u8; 6],
    pub total_settled_amount: u64,
    /// user contribution at `settled_slot`
    pub total_settled_contribution: u128,
    /// ending slot of last settled block
    pub last_settled_slot: u64,
    pub total_claimed_amount: u64,
}

impl UserRewardSettlement {
    pub fn initialize(&mut self, reward_id: u16, reward_pool_initial_slot: u64) {
        *self = Zeroable::zeroed();

        self.reward_id = reward_id;
        self.last_settled_slot = reward_pool_initial_slot;
    }

    /// Settle [RewardSettlementBlock]s from corresponding [RewardSettlement].
    ///
    /// All blocks whose ending_slot <= last_settled_slot are already settled.
    /// All the other blocks are obviously created after last update,
    /// so ending_slot >= last_updated_slot.
    ///
    /// this operation is idempotent
    pub fn settle_reward(
        &mut self,
        reward_settlement: &mut RewardSettlement,
        total_contribution_accrual_rate: u128,
        last_contribution: u128,
        last_updated_slot: u64,
    ) -> Result<()> {
        if self.last_settled_slot == reward_settlement.settlement_blocks_last_slot {
            // All settlement blocks are already settled.
            return Ok(());
        }

        let last_settled_slot = self.last_settled_slot;
        for block in reward_settlement
            .get_settlement_blocks_iter_mut()
            .skip_while(|block| block.ending_slot <= last_settled_slot)
        {
            if block.starting_slot > self.last_settled_slot {
                // There is a gap between last settled block and this block so just follow up the contribution.
                self.add_block_settled_contribution(
                    last_contribution,
                    last_updated_slot,
                    block.starting_slot,
                    total_contribution_accrual_rate,
                );
            }

            let user_block_settled_contribution = self.add_block_settled_contribution(
                last_contribution,
                last_updated_slot,
                block.ending_slot,
                total_contribution_accrual_rate,
            );
            self.total_settled_amount +=
                block.settle_user_reward(user_block_settled_contribution)?;
        }

        if self.last_settled_slot < reward_settlement.settlement_blocks_last_slot {
            // There is a gap after last settled block so just follow up the contribution.
            self.add_block_settled_contribution(
                last_contribution,
                last_updated_slot,
                reward_settlement.settlement_blocks_last_slot,
                total_contribution_accrual_rate,
            );
        }

        require_eq!(
            self.last_settled_slot,
            reward_settlement.settlement_blocks_last_slot,
        );

        Ok(())
    }

    /// We know ending_slot >= last_updated_slot,
    /// and starting_slot = last_settled_slot.
    /// So there are only two cases:
    /// case 1(last_updated_slot < last_settled_slot): updated...[starting...ending)
    /// case 2(last_updated_slot >= last_settled_slot): [starting...updated...ending)
    ///
    /// returns user_block_settled_contribution
    fn add_block_settled_contribution(
        &mut self,
        last_contribution: u128,
        last_updated_slot: u64,
        ending_slot: u64,
        total_contribution_accrual_rate: u128,
    ) -> u128 {
        let user_block_settled_contribution = last_contribution
            .saturating_sub(self.total_settled_contribution)
            + (ending_slot - self.last_settled_slot.max(last_updated_slot)) as u128
                * total_contribution_accrual_rate;

        self.total_settled_contribution += user_block_settled_contribution;
        self.last_settled_slot = ending_slot;

        user_block_settled_contribution
    }

    /// returns claimed_amount
    pub fn claim_reward(
        &mut self,
        reward_settlement: &mut RewardSettlement,
        current_slot: u64,
        amount: Option<u64>,
    ) -> Result<u64> {
        let claimable_amount = self.total_settled_amount - self.total_claimed_amount;
        let requested_amount = amount.unwrap_or(claimable_amount);

        require_gte!(
            claimable_amount,
            requested_amount,
            ErrorCode::RewardNotEnoughRewardsToClaimError,
        );

        self.total_claimed_amount += requested_amount;
        reward_settlement.claim_user_reward(requested_amount, current_slot)?;

        Ok(requested_amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settle_reward() {
        let mut reward_settlement = RewardSettlement::zeroed();
        let mut user_reward_settlement = UserRewardSettlement::zeroed();
        let mut current_slot = 0;
        let mut current_reward_pool_contribution = 0;
        let mut user_total_contribution_accrual_rate = 0;
        let mut user_last_contribution = 0;
        let mut user_last_updated_slot = 0;
        reward_settlement.initialize(0, 0, 0, current_slot);
        user_reward_settlement.initialize(0, 0);

        // new block (10, [0, 10)) at slot=10, which is immediately stale
        current_slot += 10;
        reward_settlement
            .settle_reward(10, current_reward_pool_contribution, current_slot)
            .unwrap();

        // user settled at slot=15
        current_slot += 5;
        current_reward_pool_contribution += 1_00 * 5; // 1 lamports * 5 slots
        user_reward_settlement
            .settle_reward(
                &mut reward_settlement,
                user_total_contribution_accrual_rate,
                user_last_contribution,
                user_last_updated_slot,
            )
            .unwrap();
        user_last_contribution += user_total_contribution_accrual_rate
            * (current_slot - user_last_updated_slot) as u128;
        user_last_updated_slot = current_slot;

        assert_eq!(user_reward_settlement.last_settled_slot, 10);
        assert_eq!(user_reward_settlement.total_settled_amount, 0);
        let total_settled_contribution = user_reward_settlement.total_settled_contribution;
        assert_eq!(total_settled_contribution, 0);

        // user minted 1 lamports at slot=15
        user_total_contribution_accrual_rate = 1_00; // 1 lamports

        // new block (0, [10, 20)) at slot=20
        current_slot += 5;
        current_reward_pool_contribution += 2_00 * 5; // 2 lamports * 5 slots
        reward_settlement
            .settle_reward(0, current_reward_pool_contribution, current_slot)
            .unwrap();

        // new block (10, [20, 30)) at slot=30
        current_slot += 10;
        current_reward_pool_contribution += 2_00 * 10; // 2 lamports * 10 slots
        reward_settlement
            .settle_reward(10, current_reward_pool_contribution, current_slot)
            .unwrap();

        // new block (0, [30, 40)) at slot=40
        current_slot += 10;
        current_reward_pool_contribution += 2_00 * 10; // 2 lamports * 10 slots
        reward_settlement
            .settle_reward(0, current_reward_pool_contribution, current_slot)
            .unwrap();

        // user settled at slot=45
        current_slot += 5;
        current_reward_pool_contribution += 2_00 * 5; // 2 lamports * 5 slots
        user_reward_settlement
            .settle_reward(
                &mut reward_settlement,
                user_total_contribution_accrual_rate,
                user_last_contribution,
                user_last_updated_slot,
            )
            .unwrap();
        user_last_contribution += user_total_contribution_accrual_rate
            * (current_slot - user_last_updated_slot) as u128;
        user_last_updated_slot = current_slot;

        assert_eq!(user_reward_settlement.last_settled_slot, 40);
        assert_eq!(user_reward_settlement.total_settled_amount, 5);
        let total_settled_contribution = user_reward_settlement.total_settled_contribution;
        assert_eq!(total_settled_contribution, 1_00 * 25); // slot 15 to 40

        // user burned 1 lamports at slot=45
        user_total_contribution_accrual_rate = 0; // 0 lamports

        // new block (10, [40, 50)) at slot=50
        current_slot += 5;
        current_reward_pool_contribution += 1_00 * 5; // 1 lamports * 5 slots
        reward_settlement
            .settle_reward(10, current_reward_pool_contribution, current_slot)
            .unwrap();

        // user settled at slot=50
        user_reward_settlement
            .settle_reward(
                &mut reward_settlement,
                user_total_contribution_accrual_rate,
                user_last_contribution,
                user_last_updated_slot,
            )
            .unwrap();

        assert_eq!(user_reward_settlement.last_settled_slot, 50);
        assert_eq!(user_reward_settlement.total_settled_amount, 8);
        let total_settled_contribution = user_reward_settlement.total_settled_contribution;
        assert_eq!(total_settled_contribution, 1_00 * 25 + 1_00 * 5);
    }
}
