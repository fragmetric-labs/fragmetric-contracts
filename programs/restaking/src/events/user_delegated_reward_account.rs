use anchor_lang::prelude::*;

#[event]
pub struct UserDelegatedRewardAccount {
    pub receipt_token_mint: Pubkey,
    pub user: Pubkey,
    pub user_reward_account: Pubkey,
    pub delegate: Option<Pubkey>,
}
