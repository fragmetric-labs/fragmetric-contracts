use anchor_lang::prelude::*;

#[event]
pub struct FundManagerUpdatedRewardPool {
    pub receipt_token_mint: Pubkey,
    pub reward_account_address: Pubkey,
}
