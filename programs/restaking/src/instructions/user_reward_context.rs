use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::events::UserUpdatedRewardPool;
// use crate::events::UserUpdatedRewardPool;
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
        space = std::cmp::min(8 + std::mem::size_of::<UserRewardAccount>(), 10 * 1024),
    )]
    pub user_reward_account: AccountLoader<'info, UserRewardAccount>,
}

impl<'info> UserRewardInitialContext<'info> {
    pub fn initialize_accounts(ctx: Context<Self>) -> Result<()> {
        ctx.accounts
            .user_reward_account
            .init_without_load(ctx.bumps.user_reward_account)
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
        let user_reward_account = ctx.accounts.user_reward_account.as_ref();

        let current_account_size = user_reward_account.data_len();
        let min_account_size = 8 + std::mem::size_of::<UserRewardAccount>();
        let target_account_size = desired_account_size
            .map(|desired_size| std::cmp::max(desired_size as usize, min_account_size))
            .unwrap_or(min_account_size);
        let required_realloc_size = target_account_size.saturating_sub(current_account_size);

        msg!(
            "user reward account size: current={}, target={}, required={}",
            current_account_size,
            target_account_size,
            required_realloc_size
        );

        if required_realloc_size > 0 {
            let rent = Rent::get()?;
            let current_lamports = user_reward_account.lamports();
            let minimum_lamports = rent.minimum_balance(target_account_size);
            let required_lamports = minimum_lamports.saturating_sub(current_lamports);
            if required_lamports > 0 {
                let cpi_context = CpiContext::new(
                    ctx.accounts.system_program.to_account_info(),
                    anchor_lang::system_program::Transfer {
                        from: ctx.accounts.user.to_account_info(),
                        to: user_reward_account.clone(),
                    },
                );
                anchor_lang::system_program::transfer(cpi_context, required_lamports)?;
                msg!("user reward account lamports: added={}", required_lamports);
            }

            let max_increase = solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;
            let increase = std::cmp::min(required_realloc_size, max_increase);
            let new_account_size = current_account_size + increase;
            if new_account_size < target_account_size && initialize {
                return Err(crate::errors::ErrorCode::RewardUnmetAccountReallocError)?;
            }

            user_reward_account.realloc(new_account_size, false)?;
            msg!(
                "user reward account reallocated: current={}, target={}, required={}",
                new_account_size,
                target_account_size,
                target_account_size - new_account_size
            );
        }

        if initialize {
            let receipt_token_mint = ctx.accounts.receipt_token_mint.key();
            let bump = ctx.accounts.user_reward_account.bump()?;
            let mut user_reward_account = ctx.accounts.user_reward_account.load_mut()?;

            user_reward_account.update_if_needed(bump, receipt_token_mint, ctx.accounts.user.key());

            // CHECK: won't emit empty event here for following deposit ix events' sake.
            // emit!(UserUpdatedRewardPool::new_from_initialize(
            //     receipt_token_mint,
            //     &user_reward_account,
            // ));
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

        let update = reward_account.update_user_reward_pools(&mut user_reward_account, current_slot)?;
        emit!(UserUpdatedRewardPool::new(
            ctx.accounts.receipt_token_mint.key(),
            vec![update],
        ));

        Ok(())
    }

    #[allow(unused_variables)]
    pub fn claim_rewards(ctx: Context<Self>, reward_pool_id: u8, reward_id: u8) -> Result<()> {
        ctx.accounts.check_has_one_constraints()?;

        unimplemented!()
    }
}
