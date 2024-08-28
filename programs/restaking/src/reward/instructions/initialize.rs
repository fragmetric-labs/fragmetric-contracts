use anchor_lang::prelude::*;

use crate::{common::*, constants::*, reward::*};

#[derive(Accounts)]
pub struct RewardInitialize<'info> {
    #[account(mut, address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(
        init_if_needed,
        payer = admin,
        seeds = [RewardAccount::SEED],
        bump,
        space = 8 + RewardAccount::INIT_SPACE,
    )]
    pub reward_account: Box<Account<'info, RewardAccount>>,

    pub system_program: Program<'info, System>,
}

impl<'info> RewardInitialize<'info> {
    pub fn initialize_reward(ctx: Context<RewardInitialize>) -> Result<()> {
        ctx.accounts
            .reward_account
            .initialize_if_needed(ctx.bumps.reward_account);

        Ok(())
    }
}
