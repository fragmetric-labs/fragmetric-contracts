use anchor_lang::prelude::*;

#[event]
pub struct UserUpdatedRewardPool {
    pub receipt_token_mint: Pubkey,
    pub updated_user_reward_account_addresses: Vec<Pubkey>,
}

impl UserUpdatedRewardPool {
    pub fn new(
        receipt_token_mint: Pubkey,
        updated_user_reward_account_addresses: Vec<Pubkey>,
    ) -> Self {
        Self {
            receipt_token_mint,
            updated_user_reward_account_addresses,
        }
    }

    pub fn new_from_initialize(
        receipt_token_mint: Pubkey,
        user_reward_account_address: Pubkey,
    ) -> Self {
        Self {
            receipt_token_mint,
            updated_user_reward_account_addresses: vec![user_reward_account_address],
        }
    }
}
