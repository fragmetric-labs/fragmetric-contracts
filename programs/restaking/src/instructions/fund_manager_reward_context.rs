use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};

use crate::constants::*;
use crate::events::FundManagerUpdatedRewardPool;
use crate::modules::common::PDASignerSeeds;
use crate::modules::reward::{Holder, Reward, RewardAccount, RewardPool, RewardType};

#[derive(Accounts)]
pub struct FundManagerRewardContext<'info> {
    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [RewardAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = reward_account.bump,
        has_one = receipt_token_mint,
    )]
    pub reward_account: Box<Account<'info, RewardAccount>>,

    // to optionally validate add_reward payload
    // #[account(mint::token_program = reward_token_program)]
    // pub reward_token_mint: Option<Box<InterfaceAccount<'info, Mint>>>,
    // pub reward_token_program: Option<Interface<'info, TokenInterface>>,
}

impl<'info> FundManagerRewardContext<'info> {
    pub fn add_reward_pool_holder(
        ctx: Context<Self>,
        name: String,
        description: String,
        pubkeys: Vec<Pubkey>,
    ) -> Result<()> {
        let holder = Holder::new(name, description, pubkeys)?;
        ctx.accounts.reward_account.add_holder(holder)?;

        emit!(FundManagerUpdatedRewardPool::new(
            &ctx.accounts.reward_account,
            vec![]
        ));

        Ok(())
    }

    pub fn add_reward_pool(
        ctx: Context<Self>,
        name: String,
        holder_id: Option<u8>,
        custom_contribution_accrual_rate_enabled: bool,
    ) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        let reward_pool = RewardPool::new(
            name,
            holder_id,
            custom_contribution_accrual_rate_enabled,
            current_slot,
        )?;
        let reward_pool_id = ctx.accounts.reward_account.add_reward_pool(reward_pool)?;

        emit!(FundManagerUpdatedRewardPool::new(
            &ctx.accounts.reward_account,
            vec![reward_pool_id],
        ));

        Ok(())
    }

    pub fn close_reward_pool(
        ctx: Context<Self>,
        reward_pool_id: u8,
    ) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        ctx.accounts
            .reward_account
            .close_reward_pool(reward_pool_id, current_slot)?;

        emit!(FundManagerUpdatedRewardPool::new(
            &ctx.accounts.reward_account,
            vec![reward_pool_id],
        ));

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
        bump = reward_account.bump,
        has_one = receipt_token_mint,
    )]
    pub reward_account: Box<Account<'info, RewardAccount>>,

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

        if let RewardType::Token { mint, program, decimals } = reward_type {
            let mint_account = ctx.accounts.reward_token_mint.as_ref().unwrap();
            let program_account = ctx.accounts.reward_token_program.as_ref().unwrap();
            require_keys_eq!(mint, mint_account.key());
            require_keys_eq!(program, program_account.key());
            require_eq!(decimals, mint_account.decimals);
        }

        let reward = Reward::new(
            name,
            description,
            reward_type,
        )?;
        ctx.accounts.reward_account.add_reward(reward)?;

        emit!(FundManagerUpdatedRewardPool::new(
            &ctx.accounts.reward_account,
            vec![]
        ));

        Ok(())
    }

    pub fn settle_reward(
        ctx: Context<Self>,
        reward_pool_id: u8,
        reward_id: u8,
        amount: u64,
    ) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        ctx.accounts
            .reward_account
            .settle_reward(reward_pool_id, reward_id, amount, current_slot)?;

        emit!(FundManagerUpdatedRewardPool::new(
            &ctx.accounts.reward_account,
            vec![reward_pool_id],
        ));

        Ok(())
    }
}
