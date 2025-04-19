use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::modules::reward::*;
use crate::utils::{AccountLoaderExt, PDASeeds};

#[event_cpi]
#[derive(Accounts)]
pub struct FundManagerRewardContext<'info> {
    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = reward_account.get_bump()?,
        has_one = receipt_token_mint,
        constraint = reward_account.load()?.is_latest_version() @ ErrorCode::InvalidAccountDataVersionError,
    )]
    pub reward_account: AccountLoader<'info, RewardAccount>,

    #[account(
        seeds = [RewardAccount::RESERVE_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub reward_reserve_account: SystemAccount<'info>,

    pub reward_token_mint: Option<Box<InterfaceAccount<'info, Mint>>>,

    pub reward_token_program: Option<Interface<'info, TokenInterface>>,

    pub reward_token_reserve_account: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
}
