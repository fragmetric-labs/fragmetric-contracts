use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{common::*, constants::*, reward::*};

#[derive(Accounts)]
pub struct RewardAddReward<'info> {
    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [RewardAccount::SEED],
        bump = reward_account.bump,
    )]
    pub reward_account: Box<Account<'info, RewardAccount>>,

    pub reward_token_mint: Option<Box<InterfaceAccount<'info, Mint>>>,
}

impl<'info> RewardAddReward<'info> {
    pub fn add_reward(
        ctx: Context<Self>,
        name: String,
        description: String,
        reward_type: String,
    ) -> Result<()> {
        // Verify
        require_gte!(16, name.len());
        require_gte!(128, description.len());

        let reward_type = RewardType::new(
            reward_type,
            ctx.accounts
                .reward_token_mint
                .as_ref()
                .map(|mint| mint.key()),
        )?;
        let reward = Reward::new(name, description, reward_type);
        ctx.accounts.reward_account.add_reward(reward);

        Ok(())
    }
}
