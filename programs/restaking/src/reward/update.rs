use anchor_lang::prelude::*;

use crate::{error::ErrorCode, reward::*};

impl RewardAccount {
    pub(super) fn add_holder(&mut self, mut holder: Holder) {
        holder.id = self.holders.len() as u8;
        self.holders.push(holder);
    }

    pub(super) fn add_reward(&mut self, mut reward: Reward) {
        reward.id = self.rewards.len() as u8;
        self.rewards.push(reward);
    }

    pub(super) fn add_reward_pool(&mut self, mut reward_pool: RewardPool) {
        reward_pool.id = self.reward_pools.len() as u8;
        self.reward_pools.push(reward_pool);
    }

    pub(super) fn check_pool_does_not_exist(
        &self,
        token_mint: Pubkey,
        holder_id: Option<u8>,
        custom_contribution_accrual_rate_enabled: bool,
    ) -> Result<()> {
        if self.reward_pools.iter().any(|p| {
            p.token_mint == token_mint
                && p.holder_id == holder_id
                && p.custom_contribution_accrual_rate_enabled
                    == custom_contribution_accrual_rate_enabled
        }) {
            err!(ErrorCode::RewardAlreadyExistingPool)?;
        }

        Ok(())
    }

    pub(super) fn update_reward_pools(&mut self, current_slot: u64) -> Result<()> {
        for reward_pool in &mut self.reward_pools {
            if reward_pool.closed_slot.is_none() {
                reward_pool.update_contribution(current_slot)?;
            }
            for reward_settlement in &mut reward_pool.reward_settlements {
                reward_settlement.clear_stale_settlement_blocks()?;
            }
        }

        Ok(())
    }

    pub(crate) fn update_reward_pools_token_allocation(
        &mut self,
        token_mint: Pubkey,
        amount: u64,
        contribution_accrual_rate: Option<u8>,
        from: Option<&mut UserRewardAccount>,
        to: Option<&mut UserRewardAccount>,
        current_slot: u64,
    ) -> Result<()> {
        if from.is_none() && to.is_none() || to.is_none() && contribution_accrual_rate.is_some() {
            err!(ErrorCode::TokenInvalidTransferArgs)?;
        }

        if let Some(from) = from {
            // back-fill not existing pools
            from.backfill_not_existing_pools(&self.reward_pools);
            // find "from user" related reward pools
            for reward_pool in self.get_related_pools(&from.user, &token_mint)? {
                let user_reward_pool = &mut from.user_reward_pools[reward_pool.id as usize];
                let deltas = vec![TokenAllocatedAmountDelta {
                    contribution_accrual_rate: None,
                    is_positive: false,
                    amount,
                }];

                let effective_deltas =
                    user_reward_pool.update(reward_pool, deltas, current_slot)?;
                reward_pool.update(effective_deltas, current_slot)?;
            }
        }

        if let Some(to) = to {
            // back-fill not existing pools
            to.backfill_not_existing_pools(&self.reward_pools);
            // find "to user" related reward pools
            for reward_pool in self.get_related_pools(&to.user, &token_mint)? {
                let user_reward_pool = &mut to.user_reward_pools[reward_pool.id as usize];
                let effective_contribution_accrual_rate = reward_pool
                    .custom_contribution_accrual_rate_enabled
                    .then_some(contribution_accrual_rate)
                    .flatten();
                let deltas = vec![TokenAllocatedAmountDelta {
                    contribution_accrual_rate: effective_contribution_accrual_rate,
                    is_positive: true,
                    amount,
                }];
                let effective_deltas =
                    user_reward_pool.update(reward_pool, deltas, current_slot)?;
                reward_pool.update(effective_deltas, current_slot)?;
            }
        }

        Ok(())
    }

    fn get_related_pools(
        &mut self,
        user: &Pubkey,
        token_mint: &Pubkey,
    ) -> Result<Vec<&mut RewardPool>> {
        // split into base / holder-specific pools
        let (base, holder_specific) = self
            .reward_pools
            .iter_mut()
            .filter(|p| p.token_mint == *token_mint)
            .partition::<Vec<_>, _>(|p| p.holder_id.is_none());

        // base pool should exist at least one
        if base.is_empty() {
            err!(ErrorCode::RewardInvalidPoolConfiguration)?;
        }

        // first try to find within holder specific pools
        let mut related = holder_specific
            .into_iter()
            .filter(|p| {
                self.holders
                    .get(p.holder_id.unwrap() as usize) // SAFE: partitioned by `holder_id.is_none()`
                    .unwrap() // SAFE: always checks holder existence when adding reward pool
                    .pubkeys
                    .contains(user)
            })
            .collect::<Vec<_>>();

        // otherwise falls back to base pools
        if related.is_empty() {
            related = base;
        }

        Ok(related)
    }
}

impl RewardPool {
    /// Updates the contribution of the pool into recent value.
    pub(super) fn update_contribution(&mut self, current_slot: u64) -> Result<()> {
        let elapsed_slot = current_slot
            .checked_sub(self.updated_slot)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
        let total_contribution_accrual_rate = self
            .token_allocated_amount
            .total_contribution_accrual_rate()?;
        let total_contribution = (elapsed_slot as u128)
            .checked_mul(total_contribution_accrual_rate as u128)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
        self.contribution = self
            .contribution
            .checked_add(total_contribution)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
        self.updated_slot = current_slot;

        Ok(())
    }

    fn update(
        &mut self,
        deltas: Vec<TokenAllocatedAmountDelta>,
        current_slot: u64,
    ) -> Result<Vec<TokenAllocatedAmountDelta>> {
        if self.closed_slot.is_some() {
            err!(ErrorCode::RewardPoolAlreadyClosed)?;
        }

        // First update contribution
        self.update_contribution(current_slot)?;

        // Apply deltas
        if !deltas.is_empty() {
            self.token_allocated_amount.update(deltas)
        } else {
            Ok(deltas)
        }
    }

    pub(super) fn close(&mut self, current_slot: u64) -> Result<()> {
        if self.closed_slot.is_some() {
            err!(ErrorCode::RewardPoolAlreadyClosed)?;
        }

        // First update contribution
        self.update_contribution(current_slot)?;
        self.closed_slot = Some(current_slot);
        Ok(())
    }
}

impl UserRewardAccount {
    pub(super) fn backfill_not_existing_pools(&mut self, reward_pools: &[RewardPool]) {
        let user_pool_length = self.user_reward_pools.len();
        for (i, reward_pool) in reward_pools.iter().enumerate().skip(user_pool_length) {
            self.user_reward_pools
                .push(UserRewardPool::new(i as u8, reward_pool.initial_slot));
        }
    }

    pub(super) fn update_user_reward_pools(
        &mut self,
        reward_pools: &mut [RewardPool],
        current_slot: u64,
    ) -> Result<()> {
        self.user_reward_pools
            .iter_mut()
            .zip(reward_pools.iter_mut())
            .try_for_each(|(user_reward_pool, reward_pool)| {
                user_reward_pool.update(reward_pool, vec![], current_slot)?;
                Result::Ok(())
            })
    }
}

impl UserRewardPool {
    fn update(
        &mut self,
        reward_pool: &mut RewardPool,
        deltas: Vec<TokenAllocatedAmountDelta>,
        current_slot: u64,
    ) -> Result<Vec<TokenAllocatedAmountDelta>> {
        // cache value
        let total_contribution_accrual_rate = self
            .token_allocated_amount
            .total_contribution_accrual_rate()?;

        // First update contribution, but save old data for settlement
        let last_contribution = self.contribution;
        let last_updated_slot = self.updated_slot;
        self.update_contribution(current_slot, total_contribution_accrual_rate)?;

        // Settle user reward
        for reward_settlement in &mut reward_pool.reward_settlements {
            // Find corresponding user reward settlement
            let user_reward_settlement = if let Some(user_reward_settlement) = self
                .reward_settlements
                .iter_mut()
                .find(|s| s.reward_id == reward_settlement.reward_id)
            {
                user_reward_settlement
            } else {
                let user_reward_settlement = UserRewardSettlement::new(
                    reward_settlement.reward_id,
                    reward_pool.initial_slot,
                );
                self.reward_settlements.push(user_reward_settlement);
                self.reward_settlements.last_mut().unwrap()
            };

            for block in &mut reward_settlement.settlement_blocks {
                let user_block_settled_contribution = if last_updated_slot < block.starting_slot {
                    // case 1: ...updated...[starting...ending)...
                    (block.block_height() as u128)
                        .checked_mul(total_contribution_accrual_rate as u128)
                        .ok_or_else(|| error!(ErrorCode::CalculationFailure))?
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
                        .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
                    first_half
                        .checked_add(second_half)
                        .ok_or_else(|| error!(ErrorCode::CalculationFailure))?
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
                let block_contribution = block.block_contribution();
                let user_block_settled_amount = (block_contribution > 0)
                    .then(|| {
                        u64::try_from(
                            user_block_settled_contribution
                                .checked_mul(block.amount as u128)
                                .and_then(|x| x.checked_div(block_contribution))
                                .ok_or_else(|| error!(ErrorCode::CalculationFailure))?,
                        )
                        .map_err(|_| error!(ErrorCode::CalculationFailure))
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

        self.update_total_allocated_amount(deltas)
    }

    fn update_contribution(
        &mut self,
        current_slot: u64,
        total_contribution_accrual_rate: u64, // cached
    ) -> Result<()> {
        let elapsed_slot = current_slot
            .checked_sub(self.updated_slot)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
        let total_contribution = (elapsed_slot as u128)
            .checked_mul(total_contribution_accrual_rate as u128)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;

        self.contribution = self
            .contribution
            .checked_add(total_contribution)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
        self.updated_slot = current_slot;

        Ok(())
    }

    fn update_total_allocated_amount(
        &mut self,
        deltas: Vec<TokenAllocatedAmountDelta>,
    ) -> Result<Vec<TokenAllocatedAmountDelta>> {
        if !deltas.is_empty() {
            self.token_allocated_amount.update(deltas)
        } else {
            Ok(deltas)
        }
    }
}

impl TokenAllocatedAmount {
    fn update(
        &mut self,
        deltas: Vec<TokenAllocatedAmountDelta>,
    ) -> Result<Vec<TokenAllocatedAmountDelta>> {
        let total_amount_orig = deltas.iter().try_fold(0u64, |sum, delta| {
            sum.checked_add(delta.amount)
                .ok_or_else(|| error!(ErrorCode::CalculationFailure))
        })?;

        let mut effective_deltas = vec![];
        for delta in deltas.into_iter().filter(|delta| delta.amount > 0) {
            if delta.is_positive {
                effective_deltas.push(self.add(delta)?);
            } else {
                effective_deltas.extend(self.subtract(delta)?);
            }
        }

        // Accounting: check total amount before and after
        let total_amount_effective = effective_deltas.iter().try_fold(0u64, |sum, delta| {
            sum.checked_add(delta.amount)
                .ok_or_else(|| error!(ErrorCode::CalculationFailure))
        })?;
        if total_amount_orig != total_amount_effective {
            err!(ErrorCode::RewardInvalidAccounting)?;
        }

        Ok(effective_deltas)
    }

    /// When add amount, rate = null => rate = 1.0
    fn add(&mut self, mut delta: TokenAllocatedAmountDelta) -> Result<TokenAllocatedAmountDelta> {
        delta.check_valid_addition()?;
        delta.set_default_contribution_accrual_rate();
        let contribution_accrual_rate = delta.contribution_accrual_rate.unwrap();

        if let Some(existing_record) = self
            .records
            .iter_mut()
            .find(|record| record.contribution_accrual_rate == contribution_accrual_rate)
        {
            existing_record.amount = existing_record
                .amount
                .checked_add(delta.amount)
                .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
        } else {
            self.records.push(TokenAllocatedAmountRecord {
                amount: delta.amount,
                contribution_accrual_rate,
            });
            self.records.sort_by_key(|r| r.contribution_accrual_rate);
        }
        self.total_amount = self
            .total_amount
            .checked_add(delta.amount)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;

        Ok(delta)
    }

    fn subtract(
        &mut self,
        delta: TokenAllocatedAmountDelta,
    ) -> Result<Vec<TokenAllocatedAmountDelta>> {
        delta.check_valid_subtraction()?;

        self.total_amount = self
            .total_amount
            .checked_sub(delta.amount)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;

        let mut deltas = vec![];
        if delta.contribution_accrual_rate.is_some_and(|r| r != 100) {
            let record = self
                .records
                .iter_mut()
                .find(|r| r.contribution_accrual_rate == delta.contribution_accrual_rate.unwrap())
                .ok_or_else(|| error!(ErrorCode::RewardInvalidAllocatedAmountDelta))?;
            record.amount = record
                .amount
                .checked_sub(delta.amount)
                .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
            deltas.push(delta);
        } else {
            let mut remaining_delta_amount = delta.amount;
            for record in &mut self.records {
                if remaining_delta_amount == 0 {
                    break;
                }

                let amount = std::cmp::min(record.amount, remaining_delta_amount);
                if amount > 0 {
                    record.amount -= amount;
                    remaining_delta_amount -= amount;
                    deltas.push(TokenAllocatedAmountDelta {
                        contribution_accrual_rate: Some(record.contribution_accrual_rate),
                        is_positive: false,
                        amount,
                    });
                }
            }
        }

        Ok(deltas)
    }
}

/// A change over [`TokenAllocatedAmount`].
pub struct TokenAllocatedAmountDelta {
    /// Contribution accrual rate per 1 lamports (decimals = 2)
    /// e.g., rate = 135 => actual rate = 1.35
    pub contribution_accrual_rate: Option<u8>,
    pub is_positive: bool,
    /// Nonzero - zero values are allowed but will be ignored
    pub amount: u64,
}

impl TokenAllocatedAmountDelta {
    fn check_valid_addition(&self) -> Result<()> {
        let is_contribution_accrual_rate_valid = || {
            self.contribution_accrual_rate
                .is_some_and(|rate| !(100..200).contains(&rate))
        };
        if !self.is_positive || is_contribution_accrual_rate_valid() {
            err!(ErrorCode::RewardInvalidAllocatedAmountDelta)?;
        }

        Ok(())
    }

    fn check_valid_subtraction(&self) -> Result<()> {
        if self.is_positive {
            err!(ErrorCode::RewardInvalidAllocatedAmountDelta)?;
        }

        Ok(())
    }

    fn set_default_contribution_accrual_rate(&mut self) {
        if self.contribution_accrual_rate.is_none() {
            self.contribution_accrual_rate = Some(100);
        }
    }
}
