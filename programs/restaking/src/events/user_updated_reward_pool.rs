use anchor_lang::prelude::*;

use crate::modules::reward::*;

#[event]
pub struct UserUpdatedRewardPool {
    pub receipt_token_mint: Pubkey,
    pub updates: Vec<UserRewardAccountUpdateInfo>,
}

impl UserUpdatedRewardPool {
    pub fn new(
        receipt_token_mint: Pubkey,
        updates: Vec<UserRewardAccountUpdateInfo>,
    ) -> Self {
        Self {
            receipt_token_mint,
            updates,
        }
    }

    // pub fn new_from_initialize(
    //     receipt_token_mint: Pubkey,
    //     user_reward_account: &UserRewardAccount,
    // ) -> Self {
    //     let empty_user_update = UserRewardAccountUpdateInfo::empty(user_reward_account);
    //     Self {
    //         receipt_token_mint,
    //         updates: vec![empty_user_update],
    //     }
    // }
}
