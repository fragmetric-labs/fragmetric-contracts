use anchor_lang::prelude::*;

#[event]
pub struct RewardManagerDelegatedRewardTokenAccount {
    pub vault: Pubkey,
    pub reward_manager: Pubkey,

    pub delegated_reward_token_mint: Pubkey,
    pub num_delegated_reward_token_mints: u8,
}
