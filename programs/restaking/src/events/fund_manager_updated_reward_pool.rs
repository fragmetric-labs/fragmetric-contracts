use anchor_lang::prelude::*;
use crate::modules::reward::{Holder, Reward, RewardAccount};

#[event]
pub struct FundManagerUpdatedRewardPool {
    pub receipt_token_mint: Pubkey,
    pub holders: Vec<Holder>,
    pub rewards: Vec<Reward>,
    pub updated_reward_pool_ids: Vec<u8>,
}

impl FundManagerUpdatedRewardPool {
    pub fn new_from_reward_account(
        reward_account: &RewardAccount,
        updated_reward_pool_ids: Vec<u8>,
    ) -> Self {
        Self {
            receipt_token_mint: reward_account.receipt_token_mint,
            holders: reward_account.holders.clone(),
            rewards: reward_account.rewards.clone(),
            updated_reward_pool_ids,
        }
    }
}
