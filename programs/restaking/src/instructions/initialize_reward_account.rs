use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::modules::common::{PDASignerSeeds};
use crate::modules::reward::RewardAccount;

#[derive(Accounts)]
pub struct RewardInitialize<'info> {
    #[account(mut, address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>, // fragSOL token mint account

    #[account(
        init_if_needed,
        payer = admin,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump,
        space = 10 * 1024, // desired size is: 8 + RewardAccount::INIT_SPACE
    )]
    pub reward_account: Box<Account<'info, RewardAccount>>,

    pub system_program: Program<'info, System>,
}

impl<'info> RewardInitialize<'info> {
    pub fn initialize_reward(ctx: Context<RewardInitialize>) -> Result<()> {
        ctx.accounts.reward_account.initialize_if_needed(
            ctx.bumps.reward_account,
            ctx.accounts.receipt_token_mint.key(),
        );

        Ok(())
    }
}
