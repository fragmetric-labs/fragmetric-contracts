use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::{errors::ErrorCode, modules::common::*};

use super::*;

#[constant]
/// ## Version History
/// * v_1: Initial Version
#[allow(dead_code)]
pub const USER_REWARD_ACCOUNT_CURRENT_VERSION: u16 = 1;
const USER_REWARD_ACCOUNT_REWARD_POOLS_MAX_LEN_1: usize = 4;

#[account(zero_copy)]
#[repr(C)]
pub struct UserRewardAccount {
    data_version: u16,
    pub bump: u8,
    pub receipt_token_mint: Pubkey,
    pub user: Pubkey,
    pub num_user_reward_pools: u8,
    pub max_user_reward_pools: u8,
    _padding: [u8; 11],

    user_reward_pools_1: [UserRewardPool; USER_REWARD_ACCOUNT_REWARD_POOLS_MAX_LEN_1],
}

impl PDASignerSeeds<4> for UserRewardAccount {
    const SEED: &'static [u8] = b"user_reward";

    fn signer_seeds(&self) -> [&[u8]; 4] {
        [
            Self::SEED,
            self.receipt_token_mint.as_ref(),
            self.user.as_ref(),
            self.bump_as_slice(),
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
    pub fn update_if_needed(&mut self, bump: u8, receipt_token_mint: Pubkey, user: Pubkey) {
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

    pub fn add_contribution(&mut self, contribution: u128, current_slot: u64) -> Result<()> {
        self.contribution = self
            .contribution
            .checked_add(contribution)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self.updated_slot = current_slot;

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
    fn reward_settlements_ref(&self) -> &[UserRewardSettlement] {
        &self.reward_settlements_1[..self.num_reward_settlements as usize]
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
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UserRewardAccountUpdateInfo {
    pub data_version: u16,
    pub user: Pubkey,
    pub updated_user_reward_pools: Vec<UserRewardPoolInfo>,
}

impl UserRewardAccountUpdateInfo {
    pub fn new_from_user_reward_pool(
        user_reward_account: &UserRewardAccount,
        updated_user_reward_pools: Vec<UserRewardPoolInfo>,
    ) -> Self {
        Self {
            user: user_reward_account.user,
            data_version: user_reward_account.data_version,
            updated_user_reward_pools,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UserRewardPoolInfo {
    pub token_allocated_amount: TokenAllocatedAmount,
    pub contribution: u128,
    pub updated_slot: u64,
    pub reward_pool_id: u8,
    pub reward_settlements: Vec<UserRewardSettlement>,
}

impl From<&UserRewardPool> for UserRewardPoolInfo {
    fn from(value: &UserRewardPool) -> Self {
        Self {
            token_allocated_amount: value.token_allocated_amount,
            contribution: value.contribution,
            updated_slot: value.updated_slot,
            reward_pool_id: value.reward_pool_id,
            reward_settlements: value.reward_settlements_ref().to_vec(),
        }
    }
}
