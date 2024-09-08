use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::modules::{common::*, reward::*};

// will be used only once
#[derive(Accounts)]
pub struct AdminRewardInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = payer,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump,
        space = 10 * 1024,
    )]
    pub reward_account: AccountLoader<'info, RewardAccount>,
}

impl<'info> AdminRewardInitialContext<'info> {
    pub fn initialize_accounts(ctx: Context<Self>) -> Result<()> {
        ctx.accounts
            .reward_account
            .init_without_load(ctx.bumps.reward_account)
    }
}

#[derive(Accounts)]
pub struct AdminRewardContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = reward_account.bump()?,
        // DO NOT Use has_one constraint, since reward_account is not safe yet
    )]
    pub reward_account: AccountLoader<'info, RewardAccount>,
}

impl<'info> AdminRewardContext<'info> {
    pub fn update_accounts_if_needed(
        ctx: Context<Self>,
        desired_account_size: Option<u32>,
        initialize: bool,
    ) -> Result<()> {
        let reward_account = ctx.accounts.reward_account.as_ref();

        let current_account_size = reward_account.data_len();
        let min_account_size = 8 + std::mem::size_of::<RewardAccount>();
        let target_account_size = desired_account_size
            .map(|desired_size| std::cmp::max(desired_size as usize, min_account_size))
            .unwrap_or(min_account_size);
        let required_realloc_size = target_account_size.saturating_sub(current_account_size);

        msg!(
            "reward account size: current={}, target={}, required={}",
            current_account_size,
            target_account_size,
            required_realloc_size
        );

        if required_realloc_size > 0 {
            let rent = Rent::get()?;
            let current_lamports = reward_account.lamports();
            let minimum_lamports = rent.minimum_balance(target_account_size);
            let required_lamports = minimum_lamports.saturating_sub(current_lamports);
            if required_lamports > 0 {
                let cpi_context = CpiContext::new(
                    ctx.accounts.system_program.to_account_info(),
                    anchor_lang::system_program::Transfer {
                        from: ctx.accounts.payer.to_account_info(),
                        to: reward_account.clone(),
                    },
                );
                anchor_lang::system_program::transfer(cpi_context, required_lamports)?;
                msg!("reward account lamports: added={}", required_lamports);
            }

            let max_increase = solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;
            let increase = std::cmp::min(required_realloc_size, max_increase);
            let new_account_size = current_account_size + increase;
            if new_account_size < target_account_size && initialize {
                return Err(crate::errors::ErrorCode::RewardUnmetAccountReallocError)?;
            }

            reward_account.realloc(new_account_size, false)?;
            msg!(
                "reward account reallocated: current={}, target={}, required={}",
                new_account_size,
                target_account_size,
                target_account_size - new_account_size
            );
        }

        if initialize {
            let bump = ctx.accounts.reward_account.bump()?;
            ctx.accounts
                .reward_account
                .load_mut()?
                .update_if_needed(bump, ctx.accounts.receipt_token_mint.key());
        }

        Ok(())
    }
}
