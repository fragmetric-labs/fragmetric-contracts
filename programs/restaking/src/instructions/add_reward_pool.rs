use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::events::AdminUpdatedRewardPool;
use crate::modules::common::PDASignerSeeds;
use crate::modules::reward::{RewardAccount, RewardPool};

#[derive(Accounts)]
pub struct AddRewardPoolContext<'info> {
    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = reward_account.bump,
        has_one = receipt_token_mint,
    )]
    pub reward_account: Box<Account<'info, RewardAccount>>,
}

impl<'info> AddRewardPoolContext<'info> {
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
        ctx.accounts.reward_account.check_pool_does_not_exist(
            holder_id,
            custom_contribution_accrual_rate_enabled,
        )?;

        let current_slot = Clock::get()?.slot;
        let reward_pool = RewardPool::new(
            name,
            holder_id,
            custom_contribution_accrual_rate_enabled,
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
