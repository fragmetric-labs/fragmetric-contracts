use anchor_lang::prelude::*;
use crate::modules::reward::UserRewardAccountUpdateInfo;

#[event]
pub struct UserUpdatedRewardPool {
    pub receipt_token_mint: Pubkey,
    pub updates: Vec<UserRewardAccountUpdateInfo>,
}

impl UserUpdatedRewardPool {
    pub fn new(
        receipt_token_mint: Pubkey,
        from_user_update: Option<UserRewardAccountUpdateInfo>,
        to_user_update: Option<UserRewardAccountUpdateInfo>,
    ) -> Self {
        let updates = from_user_update.into_iter().chain(to_user_update).collect();
        Self {
            receipt_token_mint,
            updates,
        }
    }
}

