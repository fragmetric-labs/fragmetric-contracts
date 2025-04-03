use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenAccount;

use crate::errors::ErrorCode;
use crate::utils::{PDASeeds, ZeroCopyHeader};

use super::*;

#[constant]
/// ## Version History
/// * v1: Initial Version (4248 ~= 4.14KB)
pub const USER_REWARD_ACCOUNT_CURRENT_VERSION: u16 = 1;
#[constant]
pub const USER_REWARD_ACCOUNT_CURRENT_SIZE: u64 =
    8 + std::mem::size_of::<UserRewardAccount>() as u64;

const USER_REWARD_ACCOUNT_REWARD_POOLS_MAX_LEN_1: usize = 4;

#[account(zero_copy)]
#[repr(C)]
pub struct UserRewardAccount {
    data_version: u16,
    bump: u8,
    pub receipt_token_mint: Pubkey,
    pub user: Pubkey,
    _padding: [u8; 13],

    base_user_reward_pool: UserRewardPool,
    bonus_user_reward_pool: UserRewardPool,
    _padding2: [u8; 1040],

    _reserved: [u8; 1040],
}

impl PDASeeds<4> for UserRewardAccount {
    const SEED: &'static [u8] = b"user_reward";

    fn get_bump(&self) -> u8 {
        self.bump
    }

    fn get_seeds(&self) -> [&[u8]; 4] {
        [
            Self::SEED,
            self.receipt_token_mint.as_ref(),
            self.user.as_ref(),
            std::slice::from_ref(&self.bump),
        ]
    }
}

impl ZeroCopyHeader for UserRewardAccount {
    fn get_bump_offset() -> usize {
        2
    }
}

impl UserRewardAccount {
    fn migrate(&mut self, bump: u8, receipt_token_mint: Pubkey, user: Pubkey) -> Result<bool> {
        let old_data_version = self.data_version;

        if self.data_version == 0 {
            self.data_version = 1;
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
            self.user = user;
        }

        // if self.data_version == 1 {
        //     self.data_version = 2;
        //     self.max_user_reward_pools += USER_REWARD_ACCOUNT_REWARD_POOLS_MAX_LEN_2;
        // }

        require_eq!(self.data_version, USER_REWARD_ACCOUNT_CURRENT_VERSION);

        Ok(old_data_version < self.data_version)
    }

    #[inline(always)]
    pub(super) fn initialize(
        &mut self,
        bump: u8,
        user_receipt_token_account: &InterfaceAccount<TokenAccount>,
    ) -> Result<bool> {
        self.migrate(
            bump,
            user_receipt_token_account.mint,
            user_receipt_token_account.owner,
        )
    }

    #[inline(always)]
    pub(super) fn update_if_needed(
        &mut self,
        user_receipt_token_account: &InterfaceAccount<TokenAccount>,
    ) -> Result<bool> {
        self.migrate(
            self.bump,
            user_receipt_token_account.mint,
            user_receipt_token_account.owner,
        )
    }

    #[inline(always)]
    pub fn is_latest_version(&self) -> bool {
        self.data_version == USER_REWARD_ACCOUNT_CURRENT_VERSION
    }

    #[inline(always)]
    pub fn is_initializing(&self) -> bool {
        self.data_version == 0
    }

    #[inline(always)]
    pub(super) fn get_user_reward_pools_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut UserRewardPool> {
        std::iter::once(&mut self.base_user_reward_pool)
            .chain(std::iter::once(&mut self.bonus_user_reward_pool))
    }

    pub(super) fn get_user_reward_pool_mut(&mut self, id: u8) -> Result<&mut UserRewardPool> {
        self.get_user_reward_pools_iter_mut()
            .nth(id as usize)
            .ok_or_else(|| error!(ErrorCode::RewardUserPoolNotFoundError))
    }

    pub(super) fn update_user_reward_pools(
        &mut self,
        reward_account: &mut RewardAccount,
        current_slot: u64,
    ) -> Result<()> {
        self.get_user_reward_pools_iter_mut()
            .zip(reward_account.get_reward_pools_iter_mut())
            .try_for_each(|(user_reward_pool, reward_pool)| {
                user_reward_pool.update_user_reward_pool(reward_pool, current_slot)
            })
    }
}
