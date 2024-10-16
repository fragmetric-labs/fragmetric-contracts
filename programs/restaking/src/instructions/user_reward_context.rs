use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::events::UserUpdatedRewardPool;
use crate::modules::reward::*;
use crate::utils::{AccountLoaderExt, PDASeeds};

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
        space = std::cmp::min(8 + std::mem::size_of::<UserRewardAccount>(), 10 * 1024),
    )]
    pub user_reward_account: AccountLoader<'info, UserRewardAccount>,
}

impl<'info> UserRewardInitialContext<'info> {
    pub fn initialize_reward_account(ctx: Context<Self>) -> Result<()> {
        ctx.accounts
            .user_reward_account
            .initialize_zero_copy_header(ctx.bumps.user_reward_account)
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
        // DO NOT Use has_one constraint, since reward_account is not safe yet
    )]
    pub user_reward_account: AccountLoader<'info, UserRewardAccount>,
}

impl<'info> UserRewardContext<'info> {
    pub fn update_accounts_if_needed(
        ctx: Context<Self>,
        desired_account_size: Option<u32>,
        initialize: bool,
    ) -> Result<()> {
        ctx.accounts
            .user_reward_account
            .expand_account_size_if_needed(
                &ctx.accounts.user,
                &ctx.accounts.system_program,
                desired_account_size,
                initialize,
            )?;

        if initialize {
            let receipt_token_mint = ctx.accounts.receipt_token_mint.key();
            let bump = ctx.accounts.user_reward_account.bump()?;
            let mut user_reward_account = ctx.accounts.user_reward_account.load_mut()?;

            user_reward_account.update_if_needed(bump, receipt_token_mint, ctx.accounts.user.key());

            emit!(UserUpdatedRewardPool::new_from_initialize(
                receipt_token_mint,
                ctx.accounts.user_reward_account.key(),
            ));
        }

        Ok(())
    }

    fn check_has_one_constraints(&self) -> Result<()> {
        require_keys_eq!(
            self.user_reward_account.load()?.receipt_token_mint,
            self.receipt_token_mint.key(),
            anchor_lang::error::ErrorCode::ConstraintHasOne,
        );
        require_keys_eq!(
            self.user_reward_account.load()?.user,
            self.user.key(),
            anchor_lang::error::ErrorCode::ConstraintHasOne,
        );

        Ok(())
    }

    pub fn update_user_reward_pools(ctx: Context<Self>) -> Result<()> {
        ctx.accounts.check_has_one_constraints()?;

        let mut reward_account = ctx.accounts.reward_account.load_mut()?;
        let mut user_reward_account = ctx.accounts.user_reward_account.load_mut()?;
        let current_slot = Clock::get()?.slot;

        reward_account.update_user_reward_pools(&mut user_reward_account, current_slot)?;

        // no events required practically...
        // emit!(UserUpdatedRewardPool::new(
        //     ctx.accounts.receipt_token_mint.key(),
        //     vec![update],
        // ));

        Ok(())
    }

    #[allow(unused_variables)]
    pub fn claim_rewards(ctx: Context<Self>, reward_pool_id: u8, reward_id: u8) -> Result<()> {
        ctx.accounts.check_has_one_constraints()?;

        unimplemented!()
    }
}
