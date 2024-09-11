use anchor_lang::prelude::*;

use crate::modules::reward::*;

#[event]
pub struct OperatorUpdatedRewardPools {
    pub receipt_token_mint: Pubkey,
    pub reward_account_address: Pubkey,
}

impl OperatorUpdatedRewardPools {
    pub fn new(reward_account: &RewardAccount, reward_account_address: Pubkey) -> Result<Self> {
        Ok(Self {
            receipt_token_mint: reward_account.receipt_token_mint,
            reward_account_address,
        })
    }
}
