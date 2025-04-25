#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

mod constants;
mod instructions;

use constants::*;
use instructions::*;

#[program]
pub mod solv {
    use super::*;

    ////////////////////////////////////////////
    // VaultInitialContext
    ////////////////////////////////////////////

    pub fn initialize_vault_account(ctx: Context<VaultAccountInitialContext>) -> Result<()> {
        ctx.accounts.vault_account.load_init()?.process_initialize(
            &ctx.accounts.vault_account.to_account_info(),
            ctx.bumps.vault_account,
            &ctx.accounts.admin,
            &ctx.accounts.delegate_reward_token_admin,
            &ctx.accounts.receipt_token_mint,
            &ctx.accounts.supported_token_mint,
            &ctx.accounts.token_program,
        )
    }

    ////////////////////////////////////////////
    // VaultRewardDelegationContext
    ////////////////////////////////////////////

    pub fn delegate_vault_reward_token_account(
        ctx: Context<VaultRewardDelegationContext>,
    ) -> Result<()> {
        ctx.accounts
            .vault_account
            .load()?
            .process_delegate_reward_token_account(
                &ctx.accounts.vault_account.to_account_info(),
                &ctx.accounts.admin,
                &ctx.accounts.delegate,
                &ctx.accounts.vault_reward_token_account,
                &ctx.accounts.token_program,
            )?;
        ctx.accounts
            .vault_account
            .load_mut()?
            .add_delegate_reward_token_account(&ctx.accounts.vault_reward_token_account)
    }
}
