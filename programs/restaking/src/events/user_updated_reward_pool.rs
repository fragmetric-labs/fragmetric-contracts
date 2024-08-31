use anchor_lang::prelude::*;
use crate::modules::reward::UserRewardAccountUpdateInfo;

#[event]
pub struct UserUpdatedRewardPool {
    pub updates: Vec<UserRewardAccountUpdateInfo>,
}

impl UserUpdatedRewardPool {
    pub fn new_from_updates(
        from_user_update: Option<UserRewardAccountUpdateInfo>,
        to_user_update: Option<UserRewardAccountUpdateInfo>,
    ) -> Self {
        let updates = from_user_update.into_iter().chain(to_user_update).collect();
        Self { updates }
    }
}

