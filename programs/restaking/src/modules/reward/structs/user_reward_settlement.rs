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
    pub fn initialize(&mut self, reward_id: u16, reward_pool_initial_slot: u64) {
        self.reward_id = reward_id;
        self.settled_amount = 0;
        self.settled_contribution = 0;
        self.settled_slot = reward_pool_initial_slot;
        self.claimed_amount = 0;
    }

    pub fn reward_id(&self) -> u16 {
        self.reward_id
    }

    pub fn add_settled_amount(&mut self, amount: u64) -> Result<()> {
        self.settled_amount = self
            .settled_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        Ok(())
    }

    pub fn settled_contribution(&self) -> u128 {
        self.settled_contribution
    }

    pub fn add_settled_contribution(&mut self, contribution: u128) -> Result<()> {
        self.settled_contribution = self
            .settled_contribution
            .checked_add(contribution)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        Ok(())
    }

    pub fn update_settled_slot(&mut self, settled_slot: u64) {
        self.settled_slot = settled_slot;
    }

    pub fn settled_slot(&self) -> u64 {
        self.settled_slot
    }
}
