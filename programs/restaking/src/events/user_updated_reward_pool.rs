use anchor_lang::prelude::*;

#[event]
pub struct UserUpdatedRewardPool {
    pub receipt_token_mint: Pubkey,
    pub updated_user_reward_accounts: Vec<Pubkey>,
}
