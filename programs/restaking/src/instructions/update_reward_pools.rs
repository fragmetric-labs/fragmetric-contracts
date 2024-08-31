use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::modules::common::PDASignerSeeds;
use crate::modules::reward::RewardAccount;

#[derive(Accounts)]
pub struct RewardUpdateRewardPools<'info> {
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

impl<'info> RewardUpdateRewardPools<'info> {
    /// (we may not need this instruction practically...)
    pub fn update_reward_pools(ctx: Context<Self>) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        ctx.accounts
            .reward_account
            .update_reward_pools(current_slot)
    }
}
