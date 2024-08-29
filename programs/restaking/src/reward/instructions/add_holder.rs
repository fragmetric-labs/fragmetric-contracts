use anchor_lang::prelude::*;

use crate::{constants::*, reward::*};

#[derive(Accounts)]
pub struct RewardAddHolder<'info> {
    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(mut, address = REWARD_ACCOUNT_ADDRESS)]
    pub reward_account: Box<Account<'info, RewardAccount>>,
}

impl<'info> RewardAddHolder<'info> {
    pub fn add_holder(
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

        Ok(())
    }
}
