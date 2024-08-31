use anchor_lang::prelude::*;

use crate::constants::*;
use crate::modules::reward::RewardAccount;

#[derive(Accounts)]
pub struct RewardInitialize<'info> {
    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(
        zero,
        address = REWARD_ACCOUNT_ADDRESS,
    )]
    pub reward_account: Box<Account<'info, RewardAccount>>,

    pub system_program: Program<'info, System>,
}

impl<'info> RewardInitialize<'info> {
    pub fn initialize_reward(ctx: Context<RewardInitialize>) -> Result<()> {
        ctx.accounts.reward_account.initialize_if_needed();

        Ok(())
    }
}
