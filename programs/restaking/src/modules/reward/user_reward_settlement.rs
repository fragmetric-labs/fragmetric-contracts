use anchor_lang::prelude::*;

use crate::errors::ErrorCode;

#[zero_copy]
#[repr(C)]
pub struct UserRewardSettlement {
    pub(super) reward_id: u16,
    _padding: [u8; 6],
    settled_amount: u64,
    pub(super) settled_contribution: u128,
    settled_slot: u64,
    claimed_amount: u64,
}

impl UserRewardSettlement {
    pub(super) fn initialize(&mut self, reward_id: u16, reward_pool_initial_slot: u64) {
        self.reward_id = reward_id;
        self.settled_amount = 0;
        self.settled_contribution = 0;
        self.settled_slot = reward_pool_initial_slot;
        self.claimed_amount = 0;
    }

    pub(super) fn settle_reward(
        &mut self,
        amount: u64,
        contribution: u128,
        settled_slot: u64,
    ) -> Result<()> {
        self.settled_amount = self
            .settled_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self.settled_contribution = self
            .settled_contribution
            .checked_add(contribution)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self.settled_slot = settled_slot;

        Ok(())
    }
}
