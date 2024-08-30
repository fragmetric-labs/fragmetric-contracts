use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{constants::*, reward::*};

#[derive(Accounts)]
pub struct RewardAddRewardPool<'info> {
    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(mut, address = REWARD_ACCOUNT_ADDRESS)]
    pub reward_account: Box<Account<'info, RewardAccount>>,

    pub token_mint: Box<InterfaceAccount<'info, Mint>>,
}

impl<'info> RewardAddRewardPool<'info> {
    pub fn add_reward_pool(
        ctx: Context<Self>,
        name: String,
        holder_id: Option<u8>,
        custom_contribution_accrual_rate_enabled: bool,
    ) -> Result<()> {
        // Verify
        require_gte!(16, name.len());
        if let Some(id) = holder_id {
            require_gt!(ctx.accounts.reward_account.holders.len(), id as usize);
        }
        let token_mint = ctx.accounts.token_mint.key();
        ctx.accounts.reward_account.check_pool_does_not_exist(
            token_mint,
            holder_id,
            custom_contribution_accrual_rate_enabled,
        )?;

        let current_slot = Clock::get()?.slot;
        let reward_pool = RewardPool::new(
            name,
            holder_id,
            custom_contribution_accrual_rate_enabled,
            token_mint,
            current_slot,
        );
        ctx.accounts.reward_account.add_reward_pool(reward_pool);

        emit!(AdminUpdatedRewardPool::new_from_reward_account(
            &ctx.accounts.reward_account,
            vec![ctx.accounts.reward_account.reward_pools.len() as u8 - 1]
        ));

        Ok(())
    }
}
