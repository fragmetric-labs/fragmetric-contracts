use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::events::AdminUpdatedRewardPool;
use crate::modules::common::PDASignerSeeds;
use crate::modules::reward::{Reward, RewardAccount, RewardType};

#[derive(Accounts)]
pub struct AddRewardContext<'info> {
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

    pub reward_token_mint: Option<Box<InterfaceAccount<'info, Mint>>>,
}

impl<'info> AddRewardContext<'info> {
    pub fn add_reward(
        ctx: Context<Self>,
        name: String,
        description: String,
        reward_type: RewardType,
    ) -> Result<()> {
        // Verify
        require_gte!(16, name.len());
        require_gte!(128, description.len());

        let reward = Reward::new(name, description, reward_type);
        ctx.accounts.reward_account.add_reward(reward);

        emit!(AdminUpdatedRewardPool::new_from_reward_account(
            &ctx.accounts.reward_account,
            vec![]
        ));

        Ok(())
    }
}
