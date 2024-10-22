use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::events;

use super::*;

pub fn process_settle_reward(
    receipt_token_mint: &InterfaceAccount<Mint>,
    reward_account: &mut AccountLoader<RewardAccount>,
    reward_pool_id: u8,
    reward_id: u16,
    amount: u64,
    current_slot: u64,
) -> Result<()> {
    reward_account
        .load_mut()?
        .settle_reward(reward_pool_id, reward_id, amount, current_slot)?;

    emit!(events::FundManagerUpdatedRewardPool {
        receipt_token_mint: receipt_token_mint.key(),
        reward_account_address: reward_account.key(),
    });

    Ok(())
}
