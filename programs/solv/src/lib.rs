#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

mod constants;
mod errors;
mod instructions;
pub mod states;

use constants::*;
use instructions::*;

#[program]
pub mod solv {
    use super::*;

    ////////////////////////////////////////////
    // VaultManagerVaultAccountInitialContext
    ////////////////////////////////////////////

    pub fn vault_manager_initialize_vault_account(
        mut ctx: Context<VaultManagerVaultAccountInitialContext>,
    ) -> Result<()> {
        process_initialize_vault_account(&mut ctx)
    }

    ////////////////////////////////////////////
    // VaultManagerVaultAccountUpdateContext
    ////////////////////////////////////////////

    pub fn vault_manager_update_vault_account_if_needed(
        mut ctx: Context<VaultManagerVaultAccountUpdateContext>,
    ) -> Result<()> {
        process_update_vault_account_if_needed(&mut ctx)
    }

    ////////////////////////////////////////////
    // VaultAdminRoleContext
    ////////////////////////////////////////////

    pub fn update_vault_admin_role(
        mut ctx: Context<VaultAdminRoleUpdateContext>,
        role: VaultAdminRole,
    ) -> Result<()> {
        process_update_vault_admin_role(&mut ctx, role)
    }

    ////////////////////////////////////////////
    // FundManagerContext
    ////////////////////////////////////////////

    pub fn fund_manager_deposit(
        mut ctx: Context<FundManagerContext>,
        vst_amount: u64,
    ) -> Result<()> {
        process_deposit(&mut ctx, vst_amount)
    }

    pub fn fund_manager_request_withdrawal(
        mut ctx: Context<FundManagerContext>,
        vrt_amount: u64,
    ) -> Result<()> {
        process_request_withdrawal(&mut ctx, vrt_amount)
    }

    pub fn fund_manager_withdraw(mut ctx: Context<FundManagerContext>) -> Result<()> {
        process_withdraw(&mut ctx)
    }

    ////////////////////////////////////////////
    // SolvManagerContext
    ////////////////////////////////////////////

    pub fn solv_manager_confirm_deposits(mut ctx: Context<SolvManagerContext>) -> Result<()> {
        process_confirm_deposits(&mut ctx)
    }

    pub fn solv_manager_complete_deposits(
        mut ctx: Context<SolvManagerContext>,
        srt_amount: u64,
        new_one_srt_as_micro_vst: u64,
    ) -> Result<()> {
        process_complete_deposits(&mut ctx, srt_amount, new_one_srt_as_micro_vst)
    }

    pub fn solv_manager_confirm_withdrawal_requests(
        mut ctx: Context<SolvManagerContext>,
    ) -> Result<()> {
        process_confirm_withdrawal_requests(&mut ctx)
    }

    pub fn solv_manager_complete_withdrawal_requests(
        mut ctx: Context<SolvManagerContext>,
        srt_amount: u64,
        vst_amount: u64,
        old_one_srt_as_micro_vst: u64,
    ) -> Result<()> {
        process_complete_withdrawal_requests(
            &mut ctx,
            srt_amount,
            vst_amount,
            old_one_srt_as_micro_vst,
        )
    }

    pub fn solv_manager_refresh_solv_receipt_token_redemption_rate(
        mut ctx: Context<SolvManagerContext>,
        new_one_srt_as_micro_vst: u64,
    ) -> Result<()> {
        process_refresh_solv_receipt_token_redemption_rate(&mut ctx, new_one_srt_as_micro_vst)
    }

    pub fn solv_manager_imply_solv_protocol_fee(
        mut ctx: Context<SolvManagerContext>,
        new_one_srt_as_micro_vst: u64,
    ) -> Result<()> {
        process_imply_solv_protocol_fee(&mut ctx, new_one_srt_as_micro_vst)
    }

    pub fn solv_manager_confirm_donations(
        mut ctx: Context<SolvManagerContext>,
        srt_amount: u64,
        vst_amount: u64,
    ) -> Result<()> {
        process_confirm_donations(&mut ctx, srt_amount, vst_amount)
    }

    ////////////////////////////////////////////
    // SolvManagerConfigurationContext
    ////////////////////////////////////////////

    // TODO/phase3: deprecate
    pub fn solv_manager_set_solv_protocol_wallet(
        mut ctx: Context<SolvManagerConfigurationContext>,
    ) -> Result<()> {
        process_set_solv_protocol_wallet(&mut ctx)
    }

    // TODO/phase3: deprecate
    pub fn solv_manager_set_solv_protocol_fee_rate(
        mut ctx: Context<SolvManagerConfigurationContext>,
        deposit_fee_rate_bps: u16,
        withdrawal_fee_rate_bps: u16,
    ) -> Result<()> {
        process_set_solv_protocol_fee_rate(&mut ctx, deposit_fee_rate_bps, withdrawal_fee_rate_bps)
    }

    ////////////////////////////////////////////
    // RewardManagerContext
    ////////////////////////////////////////////

    pub fn reward_manager_delegate_reward_token_account(
        mut ctx: Context<RewardManagerContext>,
    ) -> Result<()> {
        process_delegate_reward_token_account(&mut ctx)
    }
}
