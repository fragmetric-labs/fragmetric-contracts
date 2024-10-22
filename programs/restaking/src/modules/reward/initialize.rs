use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::utils::AccountLoaderExt;

use super::*;

pub fn process_initialize_reward_account(
    receipt_token_mint: &InterfaceAccount<Mint>,
    reward_account: &mut AccountLoader<RewardAccount>,
    bump: u8,
) -> Result<()> {
    if reward_account.as_ref().data_len() < 8 + std::mem::size_of::<RewardAccount>() {
        reward_account.initialize_zero_copy_header(bump)?;
    } else {
        reward_account
            .load_init()?
            .initialize(bump, receipt_token_mint.key());
    }
    Ok(())
}

pub fn process_initialize_user_reward_account(
    user: &Signer,
    receipt_token_mint: &InterfaceAccount<Mint>,
    user_reward_account: &mut AccountLoader<UserRewardAccount>,
    bump: u8,
) -> Result<()> {
    if user_reward_account.as_ref().data_len() < 8 + std::mem::size_of::<UserRewardAccount>() {
        user_reward_account.initialize_zero_copy_header(bump)?;
    } else {
        user_reward_account
            .load_init()?
            .initialize(bump, receipt_token_mint.key(), user.key());
    }
    Ok(())
}
