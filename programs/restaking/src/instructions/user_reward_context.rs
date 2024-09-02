use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::modules::common::PDASignerSeeds;
use crate::modules::reward::{RewardAccount, UserRewardAccount};

#[derive(Accounts)]
pub struct UserRewardContext<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,

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
        payer = user,
        seeds = [UserRewardAccount::SEED, receipt_token_mint.key().as_ref(), user.key().as_ref()],
        bump,
        space = 8 + UserRewardAccount::INIT_SPACE,
    )]
    pub user_reward_account: Box<Account<'info, UserRewardAccount>>,
}

impl<'info>  UserRewardContext<'info> {
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
        user_reward_account.update_user_reward_pools(reward_pools, current_slot)
    }

    #[allow(unused_variables)]
    pub fn claim_rewards(ctx: Context<Self>, reward_pool_id: u8, reward_id: u8) -> Result<()> {
        unimplemented!()
    }
}
