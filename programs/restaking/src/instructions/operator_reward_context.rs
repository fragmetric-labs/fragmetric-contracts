use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::modules::{common::*, reward::*};

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
        // DO NOT Use has_one constraint, since reward_account is not safe yet
    )]
    pub reward_account: AccountLoader<'info, RewardAccount>,
}

impl<'info> OperatorRewardContext<'info> {
    fn check_has_one_constraints(&self) -> Result<()> {
        require_keys_eq!(
            self.reward_account.load()?.receipt_token_mint,
            self.receipt_token_mint.key(),
            ErrorCode::ConstraintHasOne,
        );

        Ok(())
    }

    pub fn update_reward_pools(ctx: Context<Self>) -> Result<()> {
        ctx.accounts.check_has_one_constraints()?;

        let current_slot = Clock::get()?.slot;
        let mut reward_account = ctx.accounts.reward_account.load_mut()?;
        reward_account.update_reward_pools(current_slot)
    }
}
