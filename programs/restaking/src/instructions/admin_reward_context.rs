use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::modules::{common::*, reward::*};
use crate::utils::PDASeeds;

// will be used only once
#[derive(Accounts)]
pub struct AdminRewardAccountInitialContext<'info> {
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

impl<'info> AdminRewardAccountInitialContext<'info> {
    pub fn initialize_reward_account(ctx: Context<Self>) -> Result<()> {
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
        ctx.accounts.reward_account.expand_account_size_if_needed(
            &ctx.accounts.payer,
            &ctx.accounts.system_program,
            desired_account_size,
            initialize,
        )?;

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
