use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::FundManagerUpdatedRewardPool;
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
        has_one = receipt_token_mint,
    )]
    pub reward_account: AccountLoader<'info, RewardAccount>,
}

impl<'info> FundManagerRewardContext<'info> {
    pub fn add_reward_pool_holder(
        ctx: Context<Self>,
        name: String,
        description: String,
        pubkeys: Vec<Pubkey>,
    ) -> Result<()> {
        let mut reward_account = ctx.accounts.reward_account.load_mut()?;
        reward_account.add_holder(name, description, pubkeys)?;

        emit!(FundManagerUpdatedRewardPool {
            receipt_token_mint: reward_account.receipt_token_mint,
            reward_account_address: ctx.accounts.reward_account.key(),
        });

        Ok(())
    }

    pub fn add_reward_pool(
        ctx: Context<Self>,
        name: String,
        holder_id: Option<u8>,
        custom_contribution_accrual_rate_enabled: bool,
    ) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        let mut reward_account = ctx.accounts.reward_account.load_mut()?;
        reward_account.add_reward_pool(
            name,
            holder_id,
            custom_contribution_accrual_rate_enabled,
            current_slot,
        )?;

        emit!(FundManagerUpdatedRewardPool {
            receipt_token_mint: reward_account.receipt_token_mint,
            reward_account_address: ctx.accounts.reward_account.key(),
        });

        Ok(())
    }

    pub fn close_reward_pool(ctx: Context<Self>, reward_pool_id: u8) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        let mut reward_account = ctx.accounts.reward_account.load_mut()?;
        reward_account.close_reward_pool(reward_pool_id, current_slot)?;

        emit!(FundManagerUpdatedRewardPool {
            receipt_token_mint: reward_account.receipt_token_mint,
            reward_account_address: ctx.accounts.reward_account.key(),
        });

        Ok(())
    }
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
        has_one = receipt_token_mint,
    )]
    pub reward_account: AccountLoader<'info, RewardAccount>,

    #[account(mint::token_program = reward_token_program)]
    pub reward_token_mint: Option<Box<InterfaceAccount<'info, Mint>>>,
    pub reward_token_program: Option<Interface<'info, TokenInterface>>,
}

impl<'info> FundManagerRewardDistributionContext<'info> {
    pub fn add_reward(
        ctx: Context<Self>,
        name: String,
        description: String,
        reward_type: RewardType,
    ) -> Result<()> {
        if let RewardType::Token {
            mint,
            program,
            decimals,
        } = reward_type
        {
            let mint_account = ctx
                .accounts
                .reward_token_mint
                .as_ref()
                .ok_or_else(|| error!(ErrorCode::RewardInvalidRewardType))?;
            let program_account = ctx
                .accounts
                .reward_token_program
                .as_ref()
                .ok_or_else(|| error!(ErrorCode::RewardInvalidRewardType))?;
            require_keys_eq!(mint, mint_account.key(), ErrorCode::RewardInvalidRewardType);
            require_keys_eq!(
                program,
                program_account.key(),
                ErrorCode::RewardInvalidRewardType
            );
            require_eq!(
                decimals,
                mint_account.decimals,
                ErrorCode::RewardInvalidRewardType
            );
        }

        let mut reward_account = ctx.accounts.reward_account.load_mut()?;
        reward_account.add_reward(name, description, reward_type)?;

        emit!(FundManagerUpdatedRewardPool {
            receipt_token_mint: reward_account.receipt_token_mint,
            reward_account_address: ctx.accounts.reward_account.key(),
        });

        Ok(())
    }

    pub fn settle_reward(
        ctx: Context<Self>,
        reward_pool_id: u8,
        reward_id: u16,
        amount: u64,
    ) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        let mut reward_account = ctx.accounts.reward_account.load_mut()?;
        reward_account.settle_reward(reward_pool_id, reward_id, amount, current_slot)?;

        emit!(FundManagerUpdatedRewardPool {
            receipt_token_mint: reward_account.receipt_token_mint,
            reward_account_address: ctx.accounts.reward_account.key(),
        });

        Ok(())
    }
}
