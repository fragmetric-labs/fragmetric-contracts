use anchor_lang::prelude::*;
use crate::modules::reward::{RewardAccount, UserRewardAccount};

impl RewardAccount {
    pub fn initialize_if_needed(&mut self, bump: u8, receipt_token_mint: Pubkey) {
        if self.data_version == 0 {
            self.data_version = 1;
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
        }
    }
}

impl UserRewardAccount {
    pub fn initialize_if_needed(&mut self, bump: u8, receipt_token_mint: Pubkey, user: Pubkey) {
        if self.data_version == 0 {
            self.data_version = 1;
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
            self.user = user;
        }
    }
}
