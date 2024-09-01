use anchor_lang::prelude::*;
use crate::modules::reward::{Holder, Reward, RewardAccount};

#[event]
pub struct FundManagerUpdatedRewardPool {
    pub receipt_token_mint: Pubkey,
    pub updated_reward_pool_ids: Vec<u8>,

    pub holders: Vec<Holder>,
    pub rewards: Vec<Reward>,
}

impl FundManagerUpdatedRewardPool {
    pub fn new(
        reward_account: &RewardAccount,
        updated_reward_pool_ids: Vec<u8>,
    ) -> Self {
        Self {
            receipt_token_mint: reward_account.receipt_token_mint,
            updated_reward_pool_ids,
            holders: reward_account.holders.clone(),
            rewards: reward_account.rewards.clone(),
        }
    }
}
