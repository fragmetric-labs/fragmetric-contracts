use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::modules::reward::*;
use crate::utils::{AccountLoaderExt, PDASeeds};

// will be used only once
#[event_cpi]
#[derive(Accounts)]
pub struct AdminRewardAccountInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = payer,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump,
        space = std::cmp::min(
            solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE,
            8 + std::mem::size_of::<RewardAccount>(),
        ),
    )]
    pub reward_account: AccountLoader<'info, RewardAccount>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct AdminRewardAccountUpdateContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = reward_account.get_bump()?,
    )]
    pub reward_account: AccountLoader<'info, RewardAccount>,
}
