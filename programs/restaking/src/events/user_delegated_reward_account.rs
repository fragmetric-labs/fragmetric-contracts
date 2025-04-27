use anchor_lang::prelude::*;

#[event]
pub struct UserDelegatedRewardAccount {
    pub user_reward_account: Pubkey,
    pub delegate: Option<Pubkey>,
}
