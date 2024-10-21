use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::modules::reward::*;
use crate::utils::{AccountLoaderExt, PDASeeds};

#[derive(Accounts)]
pub struct FundManagerRewardContext<'info> {
    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = reward_account.bump()?,
        constraint = reward_account.load()?.is_latest_version() @ ErrorCode::InvalidDataVersionError,
    )]
    pub reward_account: AccountLoader<'info, RewardAccount>,
}

#[derive(Accounts)]
pub struct FundManagerRewardDistributionContext<'info> {
    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = reward_account.bump()?,
        constraint = reward_account.load()?.is_latest_version() @ ErrorCode::InvalidDataVersionError,
    )]
    pub reward_account: AccountLoader<'info, RewardAccount>,

    #[account(mint::token_program = reward_token_program)]
    pub reward_token_mint: Option<Box<InterfaceAccount<'info, Mint>>>,
    pub reward_token_program: Option<Interface<'info, TokenInterface>>,
}

impl<'info> FundManagerRewardDistributionContext<'info> {
    pub fn check_reward_type_constraint(&self, reward_type: &RewardType) -> Result<()> {
        if let RewardType::Token {
            mint,
            program,
            decimals,
        } = reward_type
        {
            let mint_account = self
                .reward_token_mint
                .as_ref()
                .ok_or_else(|| error!(ErrorCode::RewardInvalidRewardType))?;
            let program_account = self
                .reward_token_program
                .as_ref()
                .ok_or_else(|| error!(ErrorCode::RewardInvalidRewardType))?;
            require_keys_eq!(
                *mint,
                mint_account.key(),
                ErrorCode::RewardInvalidRewardType
            );
            require_keys_eq!(
                *program,
                program_account.key(),
                ErrorCode::RewardInvalidRewardType
            );
            require_eq!(
                *decimals,
                mint_account.decimals,
                ErrorCode::RewardInvalidRewardType
            );
        }

        Ok(())
    }
}
