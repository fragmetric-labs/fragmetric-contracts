use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::modules::common::{PDASignerSeeds};
use crate::modules::reward::RewardAccount;

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
        init_if_needed,
        payer = payer,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump,
        space = 10 * 1024, // eventually desired size is: 8 + RewardAccount::INIT_SPACE
    )]
    pub reward_account: Box<Account<'info, RewardAccount>>,
}

impl<'info> AdminRewardContext<'info> {
    pub fn initialize_reward_account_if_needed(ctx: Context<AdminRewardContext>) -> Result<()> {
        ctx.accounts.reward_account.initialize_if_needed(
            ctx.bumps.reward_account,
            ctx.accounts.receipt_token_mint.key(),
        );

        Ok(())
    }

    pub fn realloc_reward_account_if_needed(ctx: Context<Self>, desired_size: Option<u32>, asserted: bool) -> Result<()> {
        let current_size = ctx.accounts.reward_account.to_account_info().data_len();
        let min_required_size = RewardAccount::INIT_SPACE;
        let target_size = if let Some(desired_size) = desired_size {
            if desired_size as usize > min_required_size {
                desired_size as usize
            } else {
                min_required_size
            }
        } else {
            min_required_size
        };
        let remaining_increase = target_size.saturating_sub(current_size);

        msg!("reward account size: current={}, target={}, remaining={}", current_size, target_size, target_size - current_size);

        if remaining_increase > 0 {
            let rent = Rent::get()?;
            let required_lamports = rent.minimum_balance(target_size).saturating_sub(
                ctx.accounts.reward_account.to_account_info().lamports(),
            );
            if required_lamports > 0 {
                solana_program::program::invoke(
                    &solana_program::system_instruction::transfer(
                        &ctx.accounts.admin.key(),
                        &ctx.accounts.reward_account.to_account_info().key(),
                        required_lamports,
                    ),
                    &[
                        ctx.accounts.admin.to_account_info(),
                        ctx.accounts.reward_account.to_account_info(),
                        ctx.accounts.system_program.to_account_info(),
                    ],
                )?;

                let cpi_context = CpiContext::new(
                    ctx.accounts.system_program.to_account_info(),
                    anchor_lang::system_program::Transfer {
                        from: ctx.accounts.admin.to_account_info(),
                        to: ctx.accounts.reward_account.to_account_info(),
                    },
                );
                anchor_lang::system_program::transfer(cpi_context, required_lamports)?;
                msg!("reward account lamports: added={}", required_lamports);
            }

            let max_increase = solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;
            let increase = if remaining_increase > max_increase {
                if asserted {
                    return Err(crate::errors::ErrorCode::RewardUnmetAccountRealloc)?
                }
                max_increase
            } else {
                remaining_increase
            };

            let new_size = ctx.accounts.reward_account.to_account_info().data_len() + increase;
            ctx.accounts.reward_account.to_account_info().realloc(new_size, false)?;
            msg!("reward account reallocated: current={}, target={}, remaining={}", new_size, target_size, target_size - new_size);
        }

        Ok(())
    }

    pub fn update_reward_pools(ctx: Context<Self>) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        ctx.accounts
            .reward_account
            .update_reward_pools(current_slot)
    }
}
