use anchor_lang::prelude::*;

use crate::{constants::*, reward::*};

#[derive(Accounts)]
pub struct RewardCloseRewardPool<'info> {
    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(mut, address = REWARD_ACCOUNT_ADDRESS)]
    pub reward_account: Box<Account<'info, RewardAccount>>,
}

impl<'info> RewardCloseRewardPool<'info> {
    pub fn close_reward_pool(ctx: Context<Self>, reward_pool_id: u8) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        ctx.accounts
            .reward_account
            .reward_pool_mut(reward_pool_id)?
            .close(current_slot)
    }
}
