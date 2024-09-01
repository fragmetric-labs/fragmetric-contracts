use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::events::FundManagerUpdatedRewardPool;
use crate::modules::common::PDASignerSeeds;
use crate::modules::reward::RewardAccount;

#[derive(Accounts)]
pub struct RewardCloseRewardPool<'info> {
    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

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

impl<'info> RewardCloseRewardPool<'info> {
    pub fn close_reward_pool(ctx: Context<Self>, reward_pool_id: u8) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        ctx.accounts
            .reward_account
            .reward_pool_mut(reward_pool_id)?
            .close(current_slot)?;

        emit!(FundManagerUpdatedRewardPool::new_from_reward_account(
            &ctx.accounts.reward_account,
            vec![reward_pool_id],
        ));

        Ok(())
    }
}
