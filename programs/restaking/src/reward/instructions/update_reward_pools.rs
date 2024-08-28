use anchor_lang::prelude::*;

use crate::{common::*, constants::*, reward::*};

#[derive(Accounts)]
pub struct RewardUpdateRewardPools<'info> {
    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [RewardAccount::SEED],
        bump = reward_account.bump,
    )]
    pub reward_account: Box<Account<'info, RewardAccount>>,
}

impl<'info> RewardUpdateRewardPools<'info> {
    /// (we may not need this instruction practically...)
    pub fn update_reward_pools(ctx: Context<Self>) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        ctx.accounts
            .reward_account
            .update_reward_pools(current_slot)
    }
}
