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

#[account(zero_copy)]
#[repr(C)]
pub struct UserRewardAccount {
    data_version: u16,
    bump: u8,
    pub receipt_token_mint: Pubkey,
    pub user: Pubkey,
    num_user_reward_pools: u8,
    /// previous field:
    /// max_user_reward_pools: u8,
    _padding: u8,
    _reserved: [u8; 11],

    base_user_reward_pool: UserRewardPool,
    bonus_user_reward_pool: UserRewardPool,

    _reserved2: [u8; 1040],
    delegate: Pubkey,
    _reserved3: [u8; 1008],
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
    fn migrate(
        &mut self,
        bump: u8,
        receipt_token_mint: Pubkey,
        user: Pubkey,
        delegate: Option<Pubkey>,
    ) -> Result<bool> {
        let old_data_version = self.data_version;

        if self.data_version == 0 {
            self.data_version = 1;
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
            self.user = user;
            self.delegate = delegate.unwrap_or_default();
        }

        require_eq!(self.data_version, USER_REWARD_ACCOUNT_CURRENT_VERSION);

        Ok(old_data_version < self.data_version)
    }

    #[inline(always)]
    pub(super) fn initialize(
        &mut self,
        bump: u8,
        user_receipt_token_account: &InterfaceAccount<TokenAccount>,
        delegate: Option<Pubkey>,
    ) -> Result<bool> {
        self.migrate(
            bump,
            user_receipt_token_account.mint,
            user_receipt_token_account.owner,
            delegate,
        )
    }

    #[inline(always)]
    pub(super) fn update_if_needed(
        &mut self,
        user_receipt_token_account: &InterfaceAccount<TokenAccount>,
        delegate: Option<Pubkey>,
    ) -> Result<bool> {
        self.migrate(
            self.bump,
            user_receipt_token_account.mint,
            user_receipt_token_account.owner,
            delegate,
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

    /// authority = user or delegate
    pub(super) fn validate_authority(&self, authority: &Pubkey) -> Result<()> {
        if self.user != *authority && self.delegate != *authority {
            err!(ErrorCode::RewardInvalidUserRewardAccountAuthorityError)?;
        }

        Ok(())
    }

    pub(super) fn set_delegate(&mut self, delegate: Option<Pubkey>) {
        self.delegate = delegate.unwrap_or_default();
    }

    /// Must backfill not existing pools first
    #[inline(always)]
    pub(super) fn get_user_reward_pools_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut UserRewardPool> {
        [
            &mut self.base_user_reward_pool,
            &mut self.bonus_user_reward_pool,
        ]
        .into_iter()
    }

    /// Must backfill not existing pools first
    pub(super) fn get_user_reward_pool_mut(
        &mut self,
        is_bonus_pool: bool,
    ) -> Result<&mut UserRewardPool> {
        if !is_bonus_pool {
            Ok(&mut self.base_user_reward_pool)
        } else {
            Ok(&mut self.bonus_user_reward_pool)
        }
    }

    pub(super) fn backfill_not_existing_pools(
        &mut self,
        reward_account: &RewardAccount,
    ) -> Result<()> {
        // base user reward pool was previously user_reward_pools[0]
        if self.num_user_reward_pools == 0 {
            let base_reward_pool = reward_account.get_reward_pool(false)?;
            self.base_user_reward_pool
                .initialize(BASE_REWARD_POOL_ID, base_reward_pool.initial_slot)?;
            self.num_user_reward_pools = 1;
        }

        // bonus user reward pool was previously user_reward_pools[1]
        if self.num_user_reward_pools == 1 {
            let bonus_reward_pool = reward_account.get_reward_pool(true)?;
            self.bonus_user_reward_pool
                .initialize(BONUS_REWARD_POOL_ID, bonus_reward_pool.initial_slot)?;
            self.num_user_reward_pools = 2;
        }

        require_eq!(self.num_user_reward_pools, 2);

        Ok(())
    }

    pub(super) fn update_user_reward_pools(
        &mut self,
        reward_account: &mut RewardAccount,
        current_slot: u64,
    ) -> Result<()> {
        self.backfill_not_existing_pools(reward_account)?;

        self.get_user_reward_pools_iter_mut()
            .zip(reward_account.get_reward_pools_iter_mut())
            .try_for_each(|(user_reward_pool, reward_pool)| {
                user_reward_pool.update_user_reward_pool(reward_pool, current_slot)
            })
    }
}
