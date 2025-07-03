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
    /// previous fields:
    /// num_user_reward_pools: u8,
    /// max_user_reward_pools: u8,
    _padding: [u8; 2],
    _reserved: [u8; 11],

    pub(super) base_user_reward_pool: UserRewardPool,
    pub(super) bonus_user_reward_pool: UserRewardPool,

    _reserved2: [u8; 1040],

    pub(super) delegate: Pubkey,

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
        reward_account: &RewardAccount,
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

            self.base_user_reward_pool
                .initialize(&reward_account.base_reward_pool)?;
            self.bonus_user_reward_pool
                .initialize(&reward_account.bonus_reward_pool)?;

            self.delegate = delegate.unwrap_or_default();
        }

        require_eq!(self.data_version, USER_REWARD_ACCOUNT_CURRENT_VERSION);

        Ok(old_data_version < self.data_version)
    }

    #[inline(always)]
    pub(super) fn initialize(
        &mut self,
        bump: u8,
        reward_account: &RewardAccount,
        user_receipt_token_account: &TokenAccount,
        delegate: Option<Pubkey>,
    ) -> Result<bool> {
        self.migrate(
            bump,
            reward_account,
            user_receipt_token_account.mint,
            user_receipt_token_account.owner,
            delegate,
        )
    }

    #[inline(always)]
    pub(super) fn update_if_needed(
        &mut self,
        reward_account: &RewardAccount,
        user_receipt_token_account: &TokenAccount,
        delegate: Option<Pubkey>,
    ) -> Result<bool> {
        self.migrate(
            self.bump,
            reward_account,
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

    pub fn find_account_address(receipt_token_mint: &Pubkey, user: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[Self::SEED, receipt_token_mint.as_ref(), user.as_ref()],
            &crate::ID,
        )
        .0
    }

    /// authority = user or delegate (if exists)
    fn assert_authority_is_user_or_delegate(&self, authority: &Pubkey) -> Result<()> {
        #[allow(clippy::nonminimal_bool)] // is_none_or method since = 1.82.0
        if self.user != *authority
            && !self
                .get_delegate()
                .is_some_and(|delegate| delegate == authority)
        {
            err!(ErrorCode::RewardInvalidUserRewardAccountAuthorityError)?;
        }

        Ok(())
    }

    pub(super) fn get_delegate(&self) -> Option<&Pubkey> {
        (self.delegate != Pubkey::default()).then_some(&self.delegate)
    }

    pub(super) fn set_delegate(
        &mut self,
        authority: &Pubkey,
        delegate: Option<Pubkey>,
    ) -> Result<()> {
        self.assert_authority_is_user_or_delegate(authority)?;
        self.delegate = delegate.unwrap_or_default();

        Ok(())
    }

    pub fn set_delegate_unchecked(&mut self, delegate: Pubkey) {
        self.delegate = delegate;
    }

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

    pub(super) fn get_user_reward_pool_mut(&mut self, is_bonus_pool: bool) -> &mut UserRewardPool {
        if !is_bonus_pool {
            &mut self.base_user_reward_pool
        } else {
            &mut self.bonus_user_reward_pool
        }
    }

    pub(super) fn update_user_reward_pools(
        &mut self,
        reward_account: &mut RewardAccount,
        current_slot: u64,
    ) -> Result<()> {
        self.base_user_reward_pool
            .update_user_reward_pool(&mut reward_account.base_reward_pool, current_slot)?;
        self.bonus_user_reward_pool
            .update_user_reward_pool(&mut reward_account.bonus_reward_pool, current_slot)?;

        Ok(())
    }

    /// returns [claimed amount, total claimed amount]
    pub(super) fn claim_reward(
        &mut self,
        reward_account: &mut RewardAccount,
        authority: &Pubkey,
        reward_id: u16,
        is_bonus_pool: bool,
        amount: Option<u64>,
        current_slot: u64,
    ) -> Result<(u64, u64)> {
        self.assert_authority_is_user_or_delegate(authority)?;
        require_eq!(
            reward_account.get_reward(reward_id)?.claimable,
            1,
            ErrorCode::RewardNotClaimableError,
        );

        let reward_pool = reward_account.get_reward_pool_mut(is_bonus_pool);
        let user_reward_pool = self.get_user_reward_pool_mut(is_bonus_pool);

        user_reward_pool.claim_reward(reward_pool, reward_id, current_slot, amount)
    }
}
