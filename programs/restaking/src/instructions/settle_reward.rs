use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::events::AdminUpdatedRewardPool;
use crate::modules::common::PDASignerSeeds;
use crate::modules::reward::RewardAccount;

#[derive(Accounts)]
pub struct RewardSettle<'info> {
    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(mut, address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = reward_account.bump,
        has_one = receipt_token_mint,
    )]
    pub reward_account: Box<Account<'info, RewardAccount>>,
}

impl<'info> RewardSettle<'info> {
    pub fn settle_reward(
        ctx: Context<Self>,
        reward_pool_id: u8,
        reward_id: u8,
        amount: u64,
    ) -> Result<()> {
        // Verify
        require_gt!(
            ctx.accounts.reward_account.rewards.len(),
            reward_id as usize
        );

        let current_slot = Clock::get()?.slot;
        ctx.accounts
            .reward_account
            .reward_pool_mut(reward_pool_id)?
            .settle_reward(reward_id, amount, current_slot)?;

        emit!(AdminUpdatedRewardPool::new_from_reward_account(
            &ctx.accounts.reward_account,
            vec![reward_pool_id],
        ));

        Ok(())
    }
}
