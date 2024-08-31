use anchor_lang::prelude::*;

use crate::constants::*;
use crate::modules::common::PDASignerSeeds;
use crate::modules::reward::{RewardAccount, UserRewardAccount};

#[derive(Accounts)]
pub struct RewardUpdateUserRewardPools<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        seeds = [UserRewardAccount::SEED],
        bump,
        payer = user,
        space = 8 + UserRewardAccount::INIT_SPACE,
        constraint = user_reward_account.data_version == 0 || user_reward_account.user == user.key(),
    )]
    pub user_reward_account: Box<Account<'info, UserRewardAccount>>,

    // TODO Do we really need to make this account writable just to update settlement block?
    // Will it be ok for parallel execution in force-settlement-all-users situation?
    #[account(mut, address = REWARD_ACCOUNT_ADDRESS)]
    pub reward_account: Box<Account<'info, RewardAccount>>,

    pub system_program: Program<'info, System>,
}

impl<'info> RewardUpdateUserRewardPools<'info> {
    /// (we may not need this instruction practically...)
    pub fn update_user_reward_pools(ctx: Context<Self>) -> Result<()> {
        let user = ctx.accounts.user.key();
        let user_reward_account = &mut ctx.accounts.user_reward_account;
        let reward_pools = &mut ctx.accounts.reward_account.reward_pools;
        let current_slot = Clock::get()?.slot;

        user_reward_account.initialize_if_needed(ctx.bumps.user_reward_account, user);
        user_reward_account.backfill_not_existing_pools(reward_pools);
        user_reward_account.update_user_reward_pools(reward_pools, current_slot)
    }
}
