use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::modules::{common::*, reward::*};

// will be used only once
#[derive(Accounts)]
pub struct UserRewardInitialContext<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = user,
        seeds = [UserRewardAccount::SEED, receipt_token_mint.key().as_ref(), user.key().as_ref()],
        bump,
        // (when size > 10KB)
        // space = 10 * 1024,
        space = 8 + std::mem::size_of::<UserRewardAccount>(),
    )]
    pub user_reward_account: AccountLoader<'info, UserRewardAccount>,
}

impl<'info> UserRewardInitialContext<'info> {
    pub fn initialize_accounts(ctx: Context<Self>) -> Result<()> {
        // (when size > 10KB)
        // ctx.accounts.user_reward_account.init_without_load(ctx.bumps.user_reward_account)

        let mut user_reward_account = ctx.accounts.user_reward_account.load_init()?;
        user_reward_account.update_if_needed(ctx.bumps.user_reward_account, ctx.accounts.receipt_token_mint.key(), ctx.accounts.user.key());
        Ok(())
    }
}

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
        bump = reward_account.bump()?,
        has_one = receipt_token_mint,
    )]
    pub reward_account: AccountLoader<'info, RewardAccount>,

    #[account(
        mut,
        seeds = [UserRewardAccount::SEED, receipt_token_mint.key().as_ref(), user.key().as_ref()],
        bump = user_reward_account.bump()?,
        // (when size > 10KB) DO NOT Use has_one constraint, since reward_account is not safe yet
        has_one = receipt_token_mint,
        has_one = user
    )]
    pub user_reward_account: AccountLoader<'info, UserRewardAccount>,
}

impl<'info>  UserRewardContext<'info> {
    // (when size > 10KB)
    // pub fn update_accounts_if_needed(ctx: Context<Self>, desired_account_size: Option<u32>, initialize: bool) -> Result<()> {
    //     todo!();
    // }

    // (when size > 10KB)
    // fn check_has_one_constraint(&self) -> Result<()> {
    //     require_keys_eq!(
    //         self.user_reward_account.load()?.receipt_token_mint,
    //         self.receipt_token_mint.key(),
    //         anchor_lang::error::ErrorCode::ConstraintHasOne,
    //     );
    //     require_keys_eq!(
    //         self.user_reward_account.load()?.user,
    //         self.user.key(),
    //         anchor_lang::error::ErrorCode::ConstraintHasOne,
    //     );

    //     Ok(())
    // }

    pub fn update_user_reward_pools(ctx: Context<Self>) -> Result<()> {
        // (when size > 10KB)
        // ctx.accounts.check_has_one_constraint()?;

        let mut reward_account = ctx.accounts.reward_account.load_mut()?;
        let mut user_reward_account = ctx.accounts.user_reward_account.load_mut()?;
        let current_slot = Clock::get()?.slot;

        reward_account.update_user_reward_pools(&mut user_reward_account, current_slot)
    }

    #[allow(unused_variables)]
    pub fn claim_rewards(ctx: Context<Self>, reward_pool_id: u8, reward_id: u8) -> Result<()> {
        unimplemented!()
    }
}
