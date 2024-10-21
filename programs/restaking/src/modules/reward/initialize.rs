use anchor_lang::prelude::*;

use crate::utils::AccountLoaderExt;

use super::*;

pub fn initialize_reward_account(
    reward_account: &mut AccountLoader<RewardAccount>,
    receipt_token_mint: Pubkey,
    bump: u8,
) -> Result<()> {
    if reward_account.as_ref().data_len() < 8 + std::mem::size_of::<RewardAccount>() {
        reward_account.initialize_zero_copy_header(bump)?;
    } else {
        reward_account
            .load_init()?
            .initialize(bump, receipt_token_mint);
    }
    Ok(())
}

pub fn initialize_user_reward_account(
    user_reward_account: &mut AccountLoader<UserRewardAccount>,
    receipt_token_mint: Pubkey,
    user: Pubkey,
    bump: u8,
) -> Result<()> {
    if user_reward_account.as_ref().data_len() < 8 + std::mem::size_of::<UserRewardAccount>() {
        user_reward_account.initialize_zero_copy_header(bump)?;
    } else {
        user_reward_account
            .load_init()?
            .initialize(bump, receipt_token_mint, user);
    }
    Ok(())
}
