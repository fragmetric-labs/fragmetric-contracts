use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::modules::common::PDASignerSeeds;
use crate::modules::reward::RewardAccount;

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
        space = 10 * 1024,
        payer = payer,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub reward_account: Box<Account<'info, RewardAccount>>,
}

impl<'info> AdminRewardInitialContext<'info> {
    pub fn initialize_reward_account(ctx: Context<AdminRewardInitialContext>) -> Result<()> {
        let current_size = ctx.accounts.reward_account.to_account_info().data_len();
        ctx.accounts.reward_account.initialize_if_needed(
            ctx.bumps.reward_account,
            ctx.accounts.receipt_token_mint.key(),
            current_size as u32,
        );

        Ok(())
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
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = reward_account.bump,
        has_one = receipt_token_mint,
    )]
    pub reward_account: Box<Account<'info, RewardAccount>>,
}

impl<'info> AdminRewardContext<'info> {
    pub fn initialize_reward_account_if_needed(ctx: Context<Self>, desired_account_size: Option<u32>, initialize: bool) -> Result<()> {
        let current_account_size = ctx.accounts.reward_account.to_account_info().data_len();
        let min_account_size = RewardAccount::INIT_SPACE + 8;
        let target_account_size = if let Some(desired_size) = desired_account_size {
            if desired_size as usize > min_account_size {
                desired_size as usize
            } else {
                min_account_size
            }
        } else {
            min_account_size
        };
        let required_realloc_size = target_account_size.saturating_sub(current_account_size);

        msg!("reward account size: current={}, target={}, required={}", current_account_size, target_account_size, required_realloc_size);

        if required_realloc_size > 0 {
            let rent = Rent::get()?;
            let required_lamports = rent.minimum_balance(target_account_size).saturating_sub(
                ctx.accounts.reward_account.to_account_info().lamports(),
            );
            if required_lamports > 0 {
                solana_program::program::invoke(
                    &solana_program::system_instruction::transfer(
                        &ctx.accounts.payer.key(),
                        &ctx.accounts.reward_account.to_account_info().key(),
                        required_lamports,
                    ),
                    &[
                        ctx.accounts.payer.to_account_info(),
                        ctx.accounts.reward_account.to_account_info(),
                        ctx.accounts.system_program.to_account_info(),
                    ],
                )?;

                let cpi_context = CpiContext::new(
                    ctx.accounts.system_program.to_account_info(),
                    anchor_lang::system_program::Transfer {
                        from: ctx.accounts.payer.to_account_info(),
                        to: ctx.accounts.reward_account.to_account_info(),
                    },
                );
                anchor_lang::system_program::transfer(cpi_context, required_lamports)?;
                msg!("reward account lamports: added={}", required_lamports);
            }

            let max_increase = solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;
            let increase = if required_realloc_size > max_increase {
                if initialize {
                    return Err(crate::errors::ErrorCode::RewardUnmetAccountReallocError)?
                }
                max_increase
            } else {
                required_realloc_size
            };

            let new_account_size = ctx.accounts.reward_account.to_account_info().data_len() + increase;
            ctx.accounts.reward_account.to_account_info().realloc(new_account_size, false)?;
            msg!("reward account reallocated: current={}, target={}, required={}", new_account_size, target_account_size, target_account_size - new_account_size);

            if initialize {
                let bump = ctx.accounts.reward_account.bump;
                ctx.accounts.reward_account
                    .initialize_if_needed(
                        bump,
                        ctx.accounts.receipt_token_mint.key(),
                        new_account_size as u32,
                    );
            }
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
