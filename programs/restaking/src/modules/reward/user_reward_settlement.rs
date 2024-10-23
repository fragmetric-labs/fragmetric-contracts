use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::errors::ErrorCode;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct UserRewardSettlement {
    reward_id: u16,
    _padding: [u8; 6],
    settled_amount: u64,
    settled_contribution: u128,
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

    pub(super) fn reward_id(&self) -> u16 {
        self.reward_id
    }

    pub(super) fn settled_contribution(&self) -> u128 {
        self.settled_contribution
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
