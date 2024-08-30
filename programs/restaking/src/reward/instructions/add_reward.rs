use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{constants::*, reward::*};

#[derive(Accounts)]
pub struct RewardAddReward<'info> {
    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(mut, address = REWARD_ACCOUNT_ADDRESS)]
    pub reward_account: Box<Account<'info, RewardAccount>>,

    pub reward_token_mint: Option<Box<InterfaceAccount<'info, Mint>>>,
}

impl<'info> RewardAddReward<'info> {
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
