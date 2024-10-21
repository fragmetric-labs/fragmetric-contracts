use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::errors::ErrorCode;
use crate::utils::{PDASeeds, ZeroCopyHeader};

use super::*;

#[constant]
/// ## Version History
/// * v_1: Initial Version
pub const USER_REWARD_ACCOUNT_CURRENT_VERSION: u16 = 1;
const USER_REWARD_ACCOUNT_REWARD_POOLS_MAX_LEN_1: usize = 4;

#[account(zero_copy)]
#[repr(C)]
pub struct UserRewardAccount {
    data_version: u16,
    bump: u8,
    pub receipt_token_mint: Pubkey,
    pub user: Pubkey,
    pub num_user_reward_pools: u8,
    pub max_user_reward_pools: u8,
    _padding: [u8; 11],

    user_reward_pools_1: [UserRewardPool; USER_REWARD_ACCOUNT_REWARD_POOLS_MAX_LEN_1],
}

impl PDASeeds<3> for UserRewardAccount {
    const SEED: &'static [u8] = b"user_reward";

    fn seeds(&self) -> [&[u8]; 3] {
        [
            Self::SEED,
            self.receipt_token_mint.as_ref(),
            self.user.as_ref(),
        ]
    }

    fn bump_ref(&self) -> &u8 {
        &self.bump
    }
}

impl ZeroCopyHeader for UserRewardAccount {
    fn bump_offset() -> usize {
        2
    }
}

impl UserRewardAccount {
    pub fn initialize(&mut self, bump: u8, receipt_token_mint: Pubkey, user: Pubkey) {
        if self.data_version == 0 {
            self.data_version = 1;
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
            self.user = user;
            self.max_user_reward_pools = USER_REWARD_ACCOUNT_REWARD_POOLS_MAX_LEN_1 as u8;
        }

        // if self.data_version == 1 {
        //     self.data_version = 2;
        //     self.max_user_reward_pools += USER_REWARD_ACCOUNT_REWARD_POOLS_MAX_LEN_2;
        // }
    }

    pub fn update_if_needed(&mut self, receipt_token_mint: Pubkey, user: Pubkey) {
        self.initialize(self.bump, receipt_token_mint, user);
    }

    pub fn is_latest_version(&self) -> bool {
        self.data_version == USER_REWARD_ACCOUNT_CURRENT_VERSION
    }

    pub fn allocate_new_user_reward_pool(&mut self) -> Result<&mut UserRewardPool> {
        require_gt!(
            self.max_user_reward_pools,
            self.num_user_reward_pools,
            ErrorCode::RewardExceededMaxUserRewardPoolsException,
        );

        let pool = &mut self.user_reward_pools_1[self.num_user_reward_pools as usize];
        self.num_user_reward_pools += 1;

        Ok(pool)
    }

    /// How to integrate multiple fields into a single array slice or whatever...
    /// You may change the return type if needed
    fn user_reward_pools_mut(&mut self) -> &mut [UserRewardPool] {
        &mut self.user_reward_pools_1
    }

    pub fn user_reward_pools_iter_mut(&mut self) -> impl Iterator<Item = &mut UserRewardPool> {
        self.user_reward_pools_mut().iter_mut()
    }

    pub fn user_reward_pool_mut(&mut self, id: u8) -> Result<&mut UserRewardPool> {
        self.user_reward_pools_mut()
            .get_mut(id as usize)
            .ok_or_else(|| error!(ErrorCode::RewardUserPoolNotFoundError))
    }

    pub fn backfill_not_existing_pools<'a>(
        &mut self,
        reward_pools: impl Iterator<Item = &'a RewardPool>,
    ) -> Result<()> {
        let num_user_reward_pools = self.num_user_reward_pools;
        for reward_pool in reward_pools.skip(num_user_reward_pools as usize) {
            let user_reward_pool = self.allocate_new_user_reward_pool()?;
            user_reward_pool.initialize(reward_pool.id(), reward_pool.initial_slot());
        }

        Ok(())
    }
}

const USER_REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_1: usize = 16;
// const USER_REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_2: usize = 8;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct UserRewardPool {
    pub token_allocated_amount: TokenAllocatedAmount,
    contribution: u128,
    updated_slot: u64,
    reward_pool_id: u8,
    num_reward_settlements: u8,
    _padding: [u8; 6],

    _reserved: [u64; 8],

    reward_settlements_1: [UserRewardSettlement; USER_REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_1],
}

// When you want to extend user reward settlements at update v4...
// ```
// pub struct UserRewardPoolExtV4 {
//     reward_pool_id: u8,
//     num_reward_settlements: u8,
//     _padding: [u8; 14],
//     reward_settlements_2: [UserRewardSettlement; USER_REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_2],
// }
// ```
// And add new field user_reward_pools_1_ext_v4: [UserRewardPoolExtV4; USER_REWARD_ACCOUNT_REWARD_POOLS_MAX_LEN_1] to user reward account.

impl UserRewardPool {
    pub fn initialize(&mut self, reward_pool_id: u8, reward_pool_initial_slot: u64) {
        self.token_allocated_amount = TokenAllocatedAmount::zeroed();
        self.contribution = 0;
        self.updated_slot = reward_pool_initial_slot;
        self.reward_pool_id = reward_pool_id;
        self.num_reward_settlements = 0;
    }

    pub fn contribution(&self) -> u128 {
        self.contribution
    }

    /// From last updated_slot to new updated_slot
    pub fn add_contribution(&mut self, contribution: u128, updated_slot: u64) -> Result<()> {
        self.contribution = self
            .contribution
            .checked_add(contribution)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self.updated_slot = updated_slot;

        Ok(())
    }

    pub fn updated_slot(&self) -> u64 {
        self.updated_slot
    }

    pub fn allocate_new_settlement(&mut self) -> Result<&mut UserRewardSettlement> {
        require_gt!(
            USER_REWARD_POOL_REWARD_SETTLEMENTS_MAX_LEN_1,
            self.num_reward_settlements as usize,
            ErrorCode::RewardExceededMaxRewardSettlementException,
        );

        let settlement = &mut self.reward_settlements_1[self.num_reward_settlements as usize];
        self.num_reward_settlements += 1;

        Ok(settlement)
    }

    /// How to integrate multiple fields into a single array slice or whatever...
    /// You may change the return type if needed
    fn reward_settlements_mut(&mut self) -> &mut [UserRewardSettlement] {
        &mut self.reward_settlements_1[..self.num_reward_settlements as usize]
    }

    fn reward_settlements_iter_mut(&mut self) -> impl Iterator<Item = &mut UserRewardSettlement> {
        self.reward_settlements_mut().iter_mut()
    }

    pub fn reward_settlement_mut(&mut self, reward_id: u16) -> Option<&mut UserRewardSettlement> {
        self.reward_settlements_iter_mut()
            .find(|s| s.reward_id() == reward_id)
    }

    pub fn update(
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
        let last_contribution = self.contribution();
        let last_updated_slot = self.updated_slot();
        let updated_slot = reward_pool.closed_slot().unwrap_or(current_slot);
        self.update_contribution(updated_slot, total_contribution_accrual_rate)?;

        // Settle user reward
        let reward_pool_initial_slot = reward_pool.initial_slot();
        for reward_settlement in reward_pool.reward_settlements_iter_mut() {
            // Find corresponding user reward settlement
            let user_reward_settlement = if let Some(user_reward_settlement) =
                self.reward_settlement_mut(reward_settlement.reward_id())
            {
                user_reward_settlement
            } else {
                let user_reward_settlement = self.allocate_new_settlement()?;
                user_reward_settlement
                    .initialize(reward_settlement.reward_id(), reward_pool_initial_slot);
                user_reward_settlement
            };

            for block in reward_settlement.settlement_blocks_iter_mut() {
                let user_block_settled_contribution = if last_updated_slot < block.starting_slot() {
                    // case 1: ...updated...[starting...ending)...
                    (block.block_height() as u128)
                        .checked_mul(total_contribution_accrual_rate as u128)
                        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?
                } else if last_updated_slot <= block.ending_slot() {
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
                        last_contribution - user_reward_settlement.settled_contribution(); // SAFE: contribution always monotonically increase
                    let second_half = ((block.ending_slot() - last_updated_slot) as u128)
                        .checked_mul(total_contribution_accrual_rate as u128)
                        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
                    first_half
                        .checked_add(second_half)
                        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?
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
                                .checked_mul(block.amount() as u128)
                                .and_then(|x| x.checked_div(block_contribution))
                                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?,
                        )
                        .map_err(|_| error!(ErrorCode::CalculationArithmeticException))
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
                    block.ending_slot(),
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
        updated_slot: u64,
        total_contribution_accrual_rate: u64, // cached
    ) -> Result<()> {
        let elapsed_slot = updated_slot
            .checked_sub(self.updated_slot())
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        let total_contribution = (elapsed_slot as u128)
            .checked_mul(total_contribution_accrual_rate as u128)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self.add_contribution(total_contribution, updated_slot)?;

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
