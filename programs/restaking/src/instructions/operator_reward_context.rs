use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::events::OperatorUpdatedRewardPools;
use crate::modules::reward::*;
use crate::utils::{AccountLoaderExt, PDASeeds};

#[derive(Accounts)]
pub struct OperatorRewardContext<'info> {
    #[account(mut)]
    pub operator: Signer<'info>,

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
}

impl<'info> OperatorRewardContext<'info> {
    pub fn update_reward_pools(ctx: Context<Self>) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        let mut reward_account = ctx.accounts.reward_account.load_mut()?;
        reward_account.update_reward_pools(current_slot)?;

        emit!(OperatorUpdatedRewardPools::new(
            &reward_account,
            ctx.accounts.reward_account.key()
        )?);

        Ok(())
    }
}
