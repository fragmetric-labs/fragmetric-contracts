use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::events::AdminUpdatedRewardPool;
use crate::modules::common::PDASignerSeeds;
use crate::modules::reward::{Holder, RewardAccount};

#[derive(Accounts)]
pub struct AddRewardPoolHolderContext<'info> {
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

impl<'info> AddRewardPoolHolderContext<'info> {
    pub fn add_reward_pool_holder(
        ctx: Context<Self>,
        name: String,
        description: String,
        pubkeys: Vec<Pubkey>,
    ) -> Result<()> {
        // Verify
        require_gte!(16, name.len());
        require_gte!(128, description.len());
        require_gte!(20, pubkeys.len());

        let holder = Holder::new(name, description, pubkeys);
        ctx.accounts.reward_account.add_holder(holder);

        emit!(AdminUpdatedRewardPool::new_from_reward_account(
            &ctx.accounts.reward_account,
            vec![]
        ));

        Ok(())
    }
}
