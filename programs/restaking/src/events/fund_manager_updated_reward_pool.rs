use anchor_lang::prelude::*;

use crate::modules::reward::*;

#[event]
pub struct FundManagerUpdatedRewardPool {
    pub reward_account_data_version: u16,
    pub receipt_token_mint: Pubkey,
    pub updated_reward_pool_ids: Vec<u8>,

    pub holders: Vec<HolderInfo>,
    pub rewards: Vec<RewardInfo>,
}

impl FundManagerUpdatedRewardPool {
    pub fn new(reward_account: &RewardAccount, updated_reward_pool_ids: Vec<u8>) -> Result<Self> {
        let mut holders = vec![];
        for holder in reward_account.holders_iter() {
            holders.push(holder.try_into()?);
        }

        let mut rewards = vec![];
        for reward in reward_account.rewards_iter() {
            rewards.push(reward.try_into()?);
        }

        Ok(Self {
            reward_account_data_version: reward_account.data_version(),
            receipt_token_mint: reward_account.receipt_token_mint,
            updated_reward_pool_ids,
            holders,
            rewards,
        })
    }
}
