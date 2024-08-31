use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use crate::constants::*;
use crate::modules::common::PDASignerSeeds;
use crate::modules::reward::{RewardAccount, UserRewardAccount};

#[derive(Accounts)]
pub struct RewardUpdateUserRewardPools<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = reward_account.bump,
        has_one = receipt_token_mint,
    )]
    pub reward_account: Box<Account<'info, RewardAccount>>,

    #[account(
        init_if_needed,
        seeds = [UserRewardAccount::SEED, receipt_token_mint.key().as_ref(), user.key().as_ref()],
        bump,
        payer = user,
        space = 8 + UserRewardAccount::INIT_SPACE,
        constraint = user_reward_account.data_version == 0 || (user_reward_account.receipt_token_mint == receipt_token_mint.key() && user_reward_account.user == user.key()),
    )]
    pub user_reward_account: Box<Account<'info, UserRewardAccount>>,

    pub system_program: Program<'info, System>,
}

impl<'info> RewardUpdateUserRewardPools<'info> {
    /// (we may not need this instruction practically...)
    pub fn update_user_reward_pools(ctx: Context<Self>) -> Result<()> {
        let user = ctx.accounts.user.key();
        let user_reward_account = &mut ctx.accounts.user_reward_account;
        let reward_pools = &mut ctx.accounts.reward_account.reward_pools;
        let current_slot = Clock::get()?.slot;

        user_reward_account.initialize_if_needed(
            ctx.bumps.user_reward_account,
            ctx.accounts.receipt_token_mint.key(),
            user,
        );
        user_reward_account.backfill_not_existing_pools(reward_pools);
        user_reward_account.update_user_reward_pools(reward_pools, current_slot)
    }
}
