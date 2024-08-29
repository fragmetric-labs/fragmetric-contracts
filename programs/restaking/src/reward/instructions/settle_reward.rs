use anchor_lang::prelude::*;

use crate::{constants::*, reward::*};

#[derive(Accounts)]
pub struct RewardSettle<'info> {
    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(mut, address = REWARD_ACCOUNT_ADDRESS)]
    pub reward_account: Box<Account<'info, RewardAccount>>,
}

impl<'info> RewardSettle<'info> {
    pub fn settle_reward(
        ctx: Context<Self>,
        reward_pool_id: u8,
        reward_id: u8,
        amount: u64,
    ) -> Result<()> {
        // Verify
        require_gt!(
            ctx.accounts.reward_account.rewards.len(),
            reward_id as usize
        );

        let current_slot = Clock::get()?.slot;
        ctx.accounts
            .reward_account
            .reward_pool_mut(reward_pool_id)?
            .settle_reward(reward_id, amount, current_slot)
    }
}
