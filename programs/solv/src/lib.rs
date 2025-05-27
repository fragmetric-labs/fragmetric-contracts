#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

mod constants;
mod errors;
mod instructions;
mod states;

use constants::*;
use instructions::*;
use states::*;

#[program]
pub mod solv {
    use super::*;

    ////////////////////////////////////////////
    // VaultManagerVaultAccountInitialContext
    ////////////////////////////////////////////

    pub fn vault_manager_initialize_vault_account(
        ctx: Context<VaultManagerVaultAccountInitialContext>,
    ) -> Result<()> {
        process_initialize_vault_account(ctx)
    }

    ////////////////////////////////////////////
    // VaultManagerVaultAccountUpdateContext
    ////////////////////////////////////////////

    pub fn vault_manager_update_vault_account_if_needed(
        ctx: Context<VaultManagerVaultAccountUpdateContext>,
    ) -> Result<()> {
        process_update_vault_account_if_needed(ctx)
    }

    ////////////////////////////////////////////
    // VaultManagerContext
    ////////////////////////////////////////////

    // TODO/phase3: deprecate
    pub fn vault_manager_set_solv_protocol_wallet(ctx: Context<VaultManagerContext>) -> Result<()> {
        process_set_solv_protocol_wallet(ctx)
    }

    ////////////////////////////////////////////
    // VaultAdminRoleContext
    ////////////////////////////////////////////

    pub fn update_vault_admin_role(
        ctx: Context<VaultAdminRoleUpdateContext>,
        role: VaultAdminRole,
    ) -> Result<()> {
        process_update_vault_admin_role(ctx, role)
    }

    ////////////////////////////////////////////
    // FundManagerContext
    ////////////////////////////////////////////

    pub fn fund_manager_deposit(ctx: Context<FundManagerContext>, amount: u64) -> Result<()> {
        fund_manager_context::process_deposit(ctx, amount)
    }

    pub fn fund_manager_request_withdrawal(
        ctx: Context<FundManagerContext>,
        amount: u64,
    ) -> Result<()> {
        fund_manager_context::process_request_withdrawal(ctx, amount)
    }

    pub fn fund_manager_withdraw(ctx: Context<FundManagerContext>) -> Result<()> {
        fund_manager_context::process_withdraw(ctx)
    }

    ////////////////////////////////////////////
    // SolvManagerContext
    ////////////////////////////////////////////

    pub fn solv_manager_deposit(ctx: Context<SolvManagerContext>) -> Result<()> {
        solv_manager_context::process_deposit(ctx)
    }

    // TODO/phase3: deprecate
    pub fn solv_manager_confirm_deposit(
        ctx: Context<SolvManagerContext>,
        srt_amount: u64,
        srt_exchange_rate: SRTExchangeRate,
    ) -> Result<()> {
        solv_manager_context::process_confirm_deposit(ctx, srt_amount, srt_exchange_rate)
    }

    pub fn solv_manager_request_withdrawal(ctx: Context<SolvManagerContext>) -> Result<()> {
        solv_manager_context::process_request_withdrawal(ctx)
    }

    pub fn solv_manager_withdraw(
        ctx: Context<SolvManagerContext>,
        srt_amount: u64,
        vst_amount: u64,
        srt_exchange_rate: SRTExchangeRate,
    ) -> Result<()> {
        solv_manager_context::process_withdraw(ctx, srt_amount, vst_amount, srt_exchange_rate)
    }

    ////////////////////////////////////////////
    // RewardManagerContext
    ////////////////////////////////////////////

    pub fn reward_manager_delegate_reward_token_account(
        ctx: Context<RewardManagerContext>,
    ) -> Result<()> {
        process_delegate_reward_token_account(ctx)
    }
}
