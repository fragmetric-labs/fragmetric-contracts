use anchor_lang::prelude::*;

use crate::constants::*;
use crate::modules::reward::RewardAccount;

#[derive(Accounts)]
pub struct RewardUpdateRewardPools<'info> {
    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(mut, address = REWARD_ACCOUNT_ADDRESS)]
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
