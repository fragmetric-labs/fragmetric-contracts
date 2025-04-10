#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use spl_discriminator::SplDiscriminate;
use spl_transfer_hook_interface::instruction::{
    ExecuteInstruction, InitializeExtraAccountMetaListInstruction,
};

mod constants;
mod errors;
mod events;
pub mod modules;
mod utils;

mod instructions;

use constants::*;
use instructions::*;

#[program]
pub mod restaking {
    use super::*;

    ////////////////////////////////////////////
    // AdminFundAccountInitialContext
    ////////////////////////////////////////////

    pub fn admin_initialize_fund_account(
        ctx: Context<AdminFundAccountInitialContext>,
    ) -> Result<()> {
        modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_initialize_fund_account(
            &ctx.accounts.receipt_token_program,
            &ctx.accounts.admin,
            &ctx.accounts.fund_reserve_account,
            ctx.bumps.fund_account,
        )
    }

    ////////////////////////////////////////////
    // AdminFundAccountUpdateContext
    ////////////////////////////////////////////

    pub fn admin_update_fund_account_if_needed(
        ctx: Context<AdminFundAccountUpdateContext>,
        desired_account_size: Option<u32>,
    ) -> Result<()> {
        modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_update_fund_account_if_needed(
            &ctx.accounts.payer,
            &ctx.accounts.system_program,
            &ctx.accounts.fund_reserve_account,
            desired_account_size,
        )
    }

    ////////////////////////////////////////////
    // AdminFundContext
    ////////////////////////////////////////////

    pub fn admin_set_address_lookup_table_account(
        ctx: Context<AdminFundContext>,
        address_lookup_table_account: Option<Pubkey>,
    ) -> Result<()> {
        modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_set_address_lookup_table_account(address_lookup_table_account)
    }

    ////////////////////////////////////////////
    // AdminNormalizedTokenPoolInitialContext
    ////////////////////////////////////////////

    pub fn admin_initialize_normalized_token_pool_account(
        ctx: Context<AdminNormalizedTokenPoolInitialContext>,
    ) -> Result<()> {
        modules::normalization::NormalizedTokenPoolConfigurationService::new(
            &mut ctx.accounts.normalized_token_pool_account,
            &mut ctx.accounts.normalized_token_mint,
            &ctx.accounts.normalized_token_program,
        )?
        .process_initialize_normalized_token_pool_account(
            &ctx.accounts.admin,
            ctx.bumps.normalized_token_pool_account,
        )
    }

    ////////////////////////////////////////////
    // AdminNormalizedTokenPoolUpdateContext
    ////////////////////////////////////////////

    pub fn admin_update_normalized_token_pool_account_if_needed(
        ctx: Context<AdminNormalizedTokenPoolUpdateContext>,
    ) -> Result<()> {
        modules::normalization::NormalizedTokenPoolConfigurationService::new(
            &mut ctx.accounts.normalized_token_pool_account,
            &mut ctx.accounts.normalized_token_mint,
            &ctx.accounts.normalized_token_program,
        )?
        .process_update_normalized_token_pool_account_if_needed()
    }

    ////////////////////////////////////////////
    // AdminReceiptTokenMintExtraAccountMetaListInitialContext
    ////////////////////////////////////////////

    #[instruction(discriminator = InitializeExtraAccountMetaListInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn admin_initialize_extra_account_meta_list(
        ctx: Context<AdminReceiptTokenMintExtraAccountMetaListInitialContext>,
    ) -> Result<()> {
        modules::fund::FundReceiptTokenConfigurationService::new(
            &ctx.accounts.extra_account_meta_list,
        )?
        .process_initialize_extra_account_meta_list()
    }

    ////////////////////////////////////////////
    // AdminReceiptTokenMintExtraAccountMetaListUpdateContext
    ////////////////////////////////////////////

    pub fn admin_update_extra_account_meta_list_if_needed(
        ctx: Context<AdminReceiptTokenMintExtraAccountMetaListUpdateContext>,
    ) -> Result<()> {
        modules::fund::FundReceiptTokenConfigurationService::new(
            &ctx.accounts.extra_account_meta_list,
        )?
        .process_update_extra_account_meta_list_if_needed()
    }

    ////////////////////////////////////////////
    // AdminRewardAccountInitialContext
    ////////////////////////////////////////////

    pub fn admin_initialize_reward_account(
        ctx: Context<AdminRewardAccountInitialContext>,
    ) -> Result<()> {
        modules::reward::RewardConfigurationService::new(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.reward_account,
        )?
        .process_initialize_reward_account(ctx.bumps.reward_account)
    }

    ////////////////////////////////////////////
    // AdminRewardAccountUpdateContext
    ////////////////////////////////////////////

    pub fn admin_update_reward_account_if_needed(
        ctx: Context<AdminRewardAccountUpdateContext>,
        desired_account_size: Option<u32>,
    ) -> Result<()> {
        modules::reward::RewardConfigurationService::new(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.reward_account,
        )?
        .process_update_reward_account_if_needed(
            &ctx.accounts.payer,
            &ctx.accounts.system_program,
            desired_account_size,
        )
    }

    ////////////////////////////////////////////
    // AdminUserRewardAccountInitOrUpdateContext
    ////////////////////////////////////////////

    pub fn admin_create_user_reward_account_idempotent(
        ctx: Context<AdminUserRewardAccountInitOrUpdateContext>,
        desired_account_size: Option<u32>,
    ) -> Result<()> {
        let event = modules::reward::UserRewardConfigurationService::process_create_user_reward_account_idempotent(
            &ctx.accounts.system_program,
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.reward_account,
            &ctx.accounts.payer,
            &ctx.accounts.user_receipt_token_account,
            &mut ctx.accounts.user_reward_account,
            ctx.bumps.user_reward_account,
            desired_account_size
        )?;

        if let Some(event) = event {
            emit_cpi!(event);
        }

        Ok(())
    }

    ////////////////////////////////////////////
    // FundManagerFundContext
    ////////////////////////////////////////////

    pub fn fund_manager_update_fund_strategy<'info>(
        ctx: Context<'_, '_, 'info, 'info, FundManagerFundContext<'info>>,
        deposit_enabled: bool,
        donation_enabled: bool,
        withdrawal_enabled: bool,
        transfer_enabled: bool,
        withdrawal_fee_rate_bps: u16,
        withdrawal_batch_threshold_seconds: i64,
    ) -> Result<()> {
        emit_cpi!(modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_update_fund_strategy(
            deposit_enabled,
            donation_enabled,
            withdrawal_enabled,
            transfer_enabled,
            withdrawal_fee_rate_bps,
            withdrawal_batch_threshold_seconds,
        )?);

        Ok(())
    }

    pub fn fund_manager_update_sol_strategy<'info>(
        ctx: Context<'_, '_, 'info, 'info, FundManagerFundContext<'info>>,
        sol_depositable: bool,
        sol_accumulated_deposit_capacity_amount: u64,
        sol_accumulated_deposit_amount: Option<u64>,
        sol_withdrawable: bool,
        sol_withdrawal_normal_reserve_rate_bps: u16,
        sol_withdrawal_normal_reserve_max_amount: u64,
    ) -> Result<()> {
        emit_cpi!(modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_update_sol_strategy(
            sol_depositable,
            sol_accumulated_deposit_capacity_amount,
            sol_accumulated_deposit_amount,
            sol_withdrawable,
            sol_withdrawal_normal_reserve_rate_bps,
            sol_withdrawal_normal_reserve_max_amount,
        )?);

        Ok(())
    }

    pub fn fund_manager_update_supported_token_strategy<'info>(
        ctx: Context<'_, '_, 'info, 'info, FundManagerFundContext<'info>>,
        token_mint: Pubkey,
        token_depositable: bool,
        token_accumulated_deposit_capacity_amount: u64,
        token_accumulated_deposit_amount: Option<u64>,
        token_withdrawable: bool,
        token_withdrawal_normal_reserve_rate_bps: u16,
        token_withdrawal_normal_reserve_max_amount: u64,
        token_rebalancing_amount: Option<u64>,
        sol_allocation_weight: u64,
        sol_allocation_capacity_amount: u64,
    ) -> Result<()> {
        emit_cpi!(modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_update_supported_token_strategy(
            &token_mint,
            token_depositable,
            token_accumulated_deposit_capacity_amount,
            token_accumulated_deposit_amount,
            token_withdrawable,
            token_withdrawal_normal_reserve_rate_bps,
            token_withdrawal_normal_reserve_max_amount,
            token_rebalancing_amount,
            sol_allocation_weight,
            sol_allocation_capacity_amount,
        )?);

        Ok(())
    }

    pub fn fund_manager_update_restaking_vault_strategy<'info>(
        ctx: Context<'_, '_, 'info, 'info, FundManagerFundContext<'info>>,
        vault: Pubkey,
        sol_allocation_weight: u64,
        sol_allocation_capacity_amount: u64,
    ) -> Result<()> {
        emit_cpi!(modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_update_restaking_vault_strategy(
            &vault,
            sol_allocation_weight,
            sol_allocation_capacity_amount,
        )?);

        Ok(())
    }

    pub fn fund_manager_update_restaking_vault_delegation_strategy<'info>(
        ctx: Context<'_, '_, 'info, 'info, FundManagerFundContext<'info>>,
        vault: Pubkey,
        operator: Pubkey,
        token_allocation_weight: u64,
        token_allocation_capacity_amount: u64,
        token_redelegating_amount: Option<u64>,
    ) -> Result<()> {
        emit_cpi!(modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_update_restaking_vault_delegation_strategy(
            &vault,
            &operator,
            token_allocation_weight,
            token_allocation_capacity_amount,
            token_redelegating_amount,
        )?);

        Ok(())
    }

    pub fn fund_manager_add_restaking_vault_compounding_reward_token(
        ctx: Context<FundManagerFundContext>,
        vault: Pubkey,
        compounding_reward_token_mint: Pubkey,
    ) -> Result<()> {
        emit_cpi!(modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account
        )?
        .process_add_restaking_vault_compounding_reward_token(
            &vault,
            compounding_reward_token_mint,
        )?);

        Ok(())
    }

    pub fn fund_manager_add_restaking_vault_distributing_reward_token(
        ctx: Context<FundManagerFundContext>,
        vault: Pubkey,
        distributing_reward_token_mint: Pubkey,
    ) -> Result<()> {
        emit_cpi!(modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account
        )?
        .process_add_restaking_vault_distributing_reward_token(
            &vault,
            distributing_reward_token_mint
        )?);

        Ok(())
    }

    pub fn fund_manager_add_token_swap_strategy(
        ctx: Context<FundManagerFundContext>,
        from_token_mint: Pubkey,
        to_token_mint: Pubkey,
        swap_source: modules::swap::TokenSwapSource,
    ) -> Result<()> {
        emit_cpi!(modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_add_token_swap_strategy(from_token_mint, to_token_mint, swap_source)?);

        Ok(())
    }

    ////////////////////////////////////////////
    // FundManagerFundNormalizedTokenInitialContext
    ////////////////////////////////////////////

    pub fn fund_manager_initialize_fund_normalized_token<'info>(
        ctx: Context<'_, '_, 'info, 'info, FundManagerFundNormalizedTokenInitialContext<'info>>,
    ) -> Result<()> {
        emit_cpi!(modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_set_normalized_token(
            &ctx.accounts.fund_normalized_token_reserve_account,
            &ctx.accounts.normalized_token_mint,
            &ctx.accounts.normalized_token_pool_account,
            ctx.remaining_accounts,
        )?);

        Ok(())
    }

    ////////////////////////////////////////////
    // FundManagerFundWrappedTokenInitialContext
    ////////////////////////////////////////////

    pub fn fund_manager_initialize_fund_wrapped_token<'info>(
        ctx: Context<FundManagerFundWrappedTokenInitialContext>,
    ) -> Result<()> {
        emit_cpi!(modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_set_wrapped_token(
            &ctx.accounts.wrapped_token_mint,
            &ctx.accounts.admin,
            &ctx.accounts.wrapped_token_program,
            &ctx.accounts.fund_wrap_account,
            &ctx.accounts.receipt_token_wrap_account,
            &mut ctx.accounts.reward_account,
            &mut ctx.accounts.fund_wrap_account_reward_account,
        )?);

        Ok(())
    }

    ////////////////////////////////////////////
    // FundManagerFundJitoRestakingVaultInitialContext
    ////////////////////////////////////////////

    pub fn fund_manager_initialize_fund_jito_restaking_vault<'info>(
        ctx: Context<'_, '_, 'info, 'info, FundManagerFundJitoRestakingVaultInitialContext<'info>>,
    ) -> Result<()> {
        emit_cpi!(modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_add_jito_restaking_vault(
            &ctx.accounts.fund_vault_supported_token_account,
            &ctx.accounts.fund_vault_receipt_token_account,
            &ctx.accounts.vault_supported_token_mint,
            &ctx.accounts.vault_account,
            &ctx.accounts.vault_program,
            &ctx.accounts.vault_receipt_token_mint,
            ctx.remaining_accounts,
        )?);

        Ok(())
    }

    ////////////////////////////////////////////
    // FundManagerFundJitoRestakingVaultOperatorInitialContext
    ////////////////////////////////////////////

    pub fn fund_manager_initialize_fund_jito_restaking_vault_delegation<'info>(
        ctx: Context<
            '_,
            '_,
            'info,
            'info,
            FundManagerFundJitoRestakingVaultDelegationInitialContext<'info>,
        >,
    ) -> Result<()> {
        emit_cpi!(modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account
        )?
        .process_add_jito_restaking_vault_delegation(
            &ctx.accounts.vault_operator_delegation,
            &ctx.accounts.vault_account,
            &ctx.accounts.operator_account,
            ctx.remaining_accounts,
        )?);

        Ok(())
    }

    ////////////////////////////////////////////
    // FundManagerFundSupportedTokenContext
    ////////////////////////////////////////////

    pub fn fund_manager_add_supported_token<'info>(
        ctx: Context<'_, '_, 'info, 'info, FundManagerFundSupportedTokenContext<'info>>,
        pricing_source: modules::pricing::TokenPricingSource,
    ) -> Result<()> {
        emit_cpi!(modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_add_supported_token(
            &ctx.accounts.supported_token_reserve_account,
            &ctx.accounts.supported_token_mint,
            pricing_source,
            ctx.remaining_accounts,
        )?);

        Ok(())
    }

    ////////////////////////////////////////////
    // FundManagerNormalizedTokenPoolSupportedTokenContext
    ////////////////////////////////////////////

    pub fn fund_manager_add_normalized_token_pool_supported_token<'info>(
        ctx: Context<
            '_,
            '_,
            'info,
            'info,
            FundManagerNormalizedTokenPoolSupportedTokenContext<'info>,
        >,
        pricing_source: modules::pricing::TokenPricingSource,
    ) -> Result<()> {
        modules::normalization::NormalizedTokenPoolConfigurationService::new(
            &mut ctx.accounts.normalized_token_pool_account,
            &mut ctx.accounts.normalized_token_mint,
            &ctx.accounts.normalized_token_program,
        )?
        .process_add_supported_token(
            &ctx.accounts.normalized_token_pool_supported_token_account,
            &ctx.accounts.supported_token_mint,
            &ctx.accounts.supported_token_program,
            pricing_source,
            ctx.remaining_accounts,
        )?;

        Ok(())
    }

    ////////////////////////////////////////////
    // FundManagerRewardDistributionContext
    ////////////////////////////////////////////

    pub fn fund_manager_add_reward(
        ctx: Context<FundManagerRewardDistributionContext>,
        name: String,
        description: String,
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        claimable: bool,
    ) -> Result<()> {
        emit_cpi!(modules::reward::RewardConfigurationService::new(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.reward_account,
        )?
        .process_add_reward(
            ctx.accounts.reward_token_mint.as_deref(),
            ctx.accounts.reward_token_program.as_ref(),
            ctx.accounts.reward_token_reserve_account.as_deref(),
            name,
            description,
            mint,
            program,
            decimals,
            claimable,
        )?);

        Ok(())
    }

    pub fn fund_manager_update_reward(
        ctx: Context<FundManagerRewardDistributionContext>,
        reward_id: u16,
        mint: Option<Pubkey>,
        program: Option<Pubkey>,
        decimals: Option<u8>,
        claimable: bool,
    ) -> Result<()> {
        emit_cpi!(modules::reward::RewardConfigurationService::new(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.reward_account,
        )?
        .process_update_reward(
            ctx.accounts.reward_token_mint.as_deref(),
            ctx.accounts.reward_token_program.as_ref(),
            ctx.accounts.reward_token_reserve_account.as_deref(),
            reward_id,
            mint,
            program,
            decimals,
            claimable,
        )?);

        Ok(())
    }

    pub fn fund_manager_settle_reward(
        ctx: Context<FundManagerRewardDistributionContext>,
        is_bonus_pool: bool,
        reward_id: u16,
        amount: u64,
    ) -> Result<()> {
        emit_cpi!(modules::reward::RewardConfigurationService::new(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.reward_account,
        )?
        .process_settle_reward(
            ctx.accounts.reward_token_mint.as_deref(),
            ctx.accounts.reward_token_program.as_ref(),
            ctx.accounts.reward_token_reserve_account.as_deref(),
            is_bonus_pool,
            reward_id,
            amount,
        )?);

        Ok(())
    }

    ////////////////////////////////////////////
    // OperatorEmptyContext
    ////////////////////////////////////////////

    pub fn operator_log_message(
        _ctx: Context<OperatorEmptyContext>,
        message: String,
    ) -> Result<()> {
        msg!("{}", message);
        Ok(())
    }

    ////////////////////////////////////////////
    // OperatorFundContext
    ////////////////////////////////////////////

    pub fn operator_run_fund_command<'info>(
        ctx: Context<'_, '_, 'info, 'info, OperatorFundContext<'info>>,
        force_reset_command: Option<modules::fund::commands::OperationCommandEntry>,
    ) -> Result<()> {
        // check force reset command is authorized
        if let Some(command_entry) = &force_reset_command {
            // fund manager can reset the operation state anytime.
            // and admin can reset the operation state only if the command is safe.
            if !(ctx.accounts.operator.key() == FUND_MANAGER_PUBKEY
                || ctx.accounts.operator.key() == ADMIN_PUBKEY
                    && command_entry.command.is_safe_with_unchecked_params())
            {
                err!(errors::ErrorCode::FundOperationUnauthorizedCommandError)?;
            }
        }

        emit_cpi!(modules::fund::FundService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_run_command(
            &ctx.accounts.operator,
            &ctx.accounts.system_program,
            ctx.remaining_accounts,
            force_reset_command,
        )?);

        Ok(())
    }

    pub fn operator_update_fund_prices<'info>(
        ctx: Context<'_, '_, 'info, 'info, OperatorFundContext<'info>>,
    ) -> Result<()> {
        emit_cpi!(modules::fund::FundService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_update_prices(ctx.remaining_accounts,)?);

        Ok(())
    }

    pub fn operator_donate_sol_to_fund<'info>(
        ctx: Context<'_, '_, 'info, 'info, OperatorFundDonationContext<'info>>,
        amount: u64,
        offset_receivable: bool,
    ) -> Result<()> {
        emit_cpi!(modules::fund::FundService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_donate_sol(
            &ctx.accounts.operator,
            &ctx.accounts.system_program,
            &ctx.accounts.fund_reserve_account,
            ctx.remaining_accounts,
            amount,
            offset_receivable,
        )?);

        Ok(())
    }

    pub fn operator_donate_supported_token_to_fund<'info>(
        ctx: Context<'_, '_, 'info, 'info, OperatorFundSupportedTokenDonationContext<'info>>,
        amount: u64,
        offset_receivable: bool,
    ) -> Result<()> {
        emit_cpi!(modules::fund::FundService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_donate_supported_token(
            &ctx.accounts.operator,
            &ctx.accounts.supported_token_program,
            &ctx.accounts.supported_token_mint,
            &ctx.accounts.fund_supported_token_reserve_account,
            &ctx.accounts.operator_supported_token_account,
            ctx.remaining_accounts,
            amount,
            offset_receivable,
        )?);

        Ok(())
    }

    ////////////////////////////////////////////
    // OperatorRewardContext
    ////////////////////////////////////////////

    pub fn operator_update_reward_pools(ctx: Context<OperatorRewardContext>) -> Result<()> {
        emit_cpi!(modules::reward::RewardService::new(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.reward_account,
        )?
        .process_update_reward_pools()?);

        Ok(())
    }

    ////////////////////////////////////////////
    // OperatorNormalizedTokenPoolContext
    ////////////////////////////////////////////

    pub fn operator_update_normalized_token_pool_prices<'info>(
        ctx: Context<'_, '_, 'info, 'info, OperatorNormalizedTokenPoolContext<'info>>,
    ) -> Result<()> {
        emit_cpi!(modules::normalization::NormalizedTokenPoolService::new(
            &mut ctx.accounts.normalized_token_pool_account,
            &mut ctx.accounts.normalized_token_mint,
            &ctx.accounts.normalized_token_program,
        )?
        .process_update_prices(ctx.remaining_accounts)?);

        Ok(())
    }

    ////////////////////////////////////////////
    // SlasherNormalizedTokenWithdrawalAccountInitialContext
    ////////////////////////////////////////////

    // TODO: untested
    pub fn slasher_initialize_normalized_token_withdrawal_account<'info>(
        ctx: Context<
            '_,
            '_,
            'info,
            'info,
            SlasherNormalizedTokenWithdrawalAccountInitialContext<'info>,
        >,
    ) -> Result<()> {
        modules::normalization::NormalizedTokenPoolService::new(
            &mut ctx.accounts.normalized_token_pool_account,
            &mut ctx.accounts.normalized_token_mint,
            &ctx.accounts.normalized_token_program,
        )?
        .process_initialize_withdrawal_account(
            &mut ctx
                .accounts
                .slasher_normalized_token_withdrawal_ticket_account,
            ctx.bumps.slasher_normalized_token_withdrawal_ticket_account,
            &mut ctx.accounts.slasher_normalized_token_account,
            &mut ctx.accounts.slasher,
            ctx.remaining_accounts,
        )?;

        Ok(())
    }

    ////////////////////////////////////////////
    // SlasherNormalizedTokenWithdrawContext
    ////////////////////////////////////////////

    // TODO: untested
    pub fn slasher_withdraw_normalized_token(
        ctx: Context<SlasherNormalizedTokenWithdrawContext>,
    ) -> Result<()> {
        modules::normalization::NormalizedTokenPoolService::new(
            &mut ctx.accounts.normalized_token_pool_account,
            &mut ctx.accounts.normalized_token_mint,
            &ctx.accounts.normalized_token_program,
        )?
        .process_withdraw(
            &ctx.accounts.supported_token_mint,
            &ctx.accounts.supported_token_program,
            &mut ctx
                .accounts
                .normalized_token_pool_supported_token_reserve_account,
            &mut ctx
                .accounts
                .slasher_normalized_token_withdrawal_ticket_account,
            &ctx.accounts.slasher,
            &mut ctx.accounts.destination_supported_token_account,
            &mut ctx.accounts.destination_rent_lamports_account,
        )?;

        Ok(())
    }

    ////////////////////////////////////////////
    // UserFundAccountInitOrUpdateContext
    ////////////////////////////////////////////
    pub fn user_create_fund_account_idempotent(
        ctx: Context<UserFundAccountInitOrUpdateContext>,
        desired_account_size: Option<u32>,
    ) -> Result<()> {
        let event = modules::fund::UserFundConfigurationService::process_create_user_fund_account_idempotent(
            &ctx.accounts.system_program,
            &mut ctx.accounts.receipt_token_mint,
            &ctx.accounts.user,
            &ctx.accounts.user_receipt_token_account,
            &mut ctx.accounts.user_fund_account,
            ctx.bumps.user_fund_account,
            desired_account_size,
        )?;

        if let Some(event) = event {
            emit_cpi!(event);
        }

        Ok(())
    }

    ////////////////////////////////////////////
    // UserFundContext
    ////////////////////////////////////////////

    pub fn user_deposit_sol<'info>(
        ctx: Context<'_, '_, 'info, 'info, UserFundContext<'info>>,
        amount: u64,
        metadata: Option<modules::fund::DepositMetadata>,
    ) -> Result<()> {
        emit_cpi!(modules::fund::UserFundService::new(
            &mut ctx.accounts.receipt_token_mint,
            &ctx.accounts.receipt_token_program,
            &mut ctx.accounts.fund_account,
            &mut ctx.accounts.reward_account,
            &ctx.accounts.user,
            &mut ctx.accounts.user_receipt_token_account,
            &mut ctx.accounts.user_fund_account,
            &mut ctx.accounts.user_reward_account,
        )?
        .process_deposit_sol(
            &ctx.accounts.system_program,
            &ctx.accounts.fund_reserve_account,
            &ctx.accounts.instructions_sysvar,
            ctx.remaining_accounts,
            amount,
            metadata,
            &ADMIN_PUBKEY,
        )?);

        Ok(())
    }

    pub fn user_request_withdrawal<'info>(
        ctx: Context<'_, '_, 'info, 'info, UserFundContext<'info>>,
        receipt_token_amount: u64,
        supported_token_mint: Option<Pubkey>,
    ) -> Result<()> {
        emit_cpi!(modules::fund::UserFundService::new(
            &mut ctx.accounts.receipt_token_mint,
            &ctx.accounts.receipt_token_program,
            &mut ctx.accounts.fund_account,
            &mut ctx.accounts.reward_account,
            &ctx.accounts.user,
            &mut ctx.accounts.user_receipt_token_account,
            &mut ctx.accounts.user_fund_account,
            &mut ctx.accounts.user_reward_account,
        )?
        .process_request_withdrawal(
            &mut ctx.accounts.receipt_token_lock_account,
            supported_token_mint,
            ctx.remaining_accounts,
            receipt_token_amount,
        )?);

        Ok(())
    }

    pub fn user_cancel_withdrawal_request<'info>(
        ctx: Context<'_, '_, 'info, 'info, UserFundContext<'info>>,
        request_id: u64,
        supported_token_mint: Option<Pubkey>,
    ) -> Result<()> {
        emit_cpi!(modules::fund::UserFundService::new(
            &mut ctx.accounts.receipt_token_mint,
            &ctx.accounts.receipt_token_program,
            &mut ctx.accounts.fund_account,
            &mut ctx.accounts.reward_account,
            &ctx.accounts.user,
            &mut ctx.accounts.user_receipt_token_account,
            &mut ctx.accounts.user_fund_account,
            &mut ctx.accounts.user_reward_account,
        )?
        .process_cancel_withdrawal_request(
            &mut ctx.accounts.receipt_token_lock_account,
            ctx.remaining_accounts,
            request_id,
            supported_token_mint,
        )?);

        Ok(())
    }

    pub fn user_withdraw_sol(
        ctx: Context<UserFundWithdrawContext>,
        _batch_id: u64,
        request_id: u64,
    ) -> Result<()> {
        emit_cpi!(modules::fund::UserFundService::new(
            &mut ctx.accounts.receipt_token_mint,
            &ctx.accounts.receipt_token_program,
            &mut ctx.accounts.fund_account,
            &mut ctx.accounts.reward_account,
            &ctx.accounts.user,
            &mut ctx.accounts.user_receipt_token_account,
            &mut ctx.accounts.user_fund_account,
            &mut ctx.accounts.user_reward_account,
        )?
        .process_withdraw_sol(
            &ctx.accounts.system_program,
            &mut ctx.accounts.fund_withdrawal_batch_account,
            &ctx.accounts.fund_reserve_account,
            &ctx.accounts.fund_treasury_account,
            request_id,
        )?);

        Ok(())
    }

    ////////////////////////////////////////////
    // UserFundSupportedTokenContext
    ////////////////////////////////////////////

    pub fn user_deposit_supported_token<'info>(
        ctx: Context<'_, '_, 'info, 'info, UserFundSupportedTokenContext<'info>>,
        amount: u64,
        metadata: Option<modules::fund::DepositMetadata>,
    ) -> Result<()> {
        emit_cpi!(modules::fund::UserFundService::new(
            &mut ctx.accounts.receipt_token_mint,
            &ctx.accounts.receipt_token_program,
            &mut ctx.accounts.fund_account,
            &mut ctx.accounts.reward_account,
            &ctx.accounts.user,
            &mut ctx.accounts.user_receipt_token_account,
            &mut ctx.accounts.user_fund_account,
            &mut ctx.accounts.user_reward_account,
        )?
        .process_deposit_supported_token(
            &ctx.accounts.supported_token_program,
            &ctx.accounts.supported_token_mint,
            &ctx.accounts.fund_supported_token_reserve_account,
            &ctx.accounts.user_supported_token_account,
            &ctx.accounts.instructions_sysvar,
            ctx.remaining_accounts,
            amount,
            metadata,
            &ADMIN_PUBKEY,
        )?);

        Ok(())
    }

    pub fn user_withdraw_supported_token(
        ctx: Context<UserFundWithdrawSupportedTokenContext>,
        _batch_id: u64,
        request_id: u64,
    ) -> Result<()> {
        emit_cpi!(modules::fund::UserFundService::new(
            &mut ctx.accounts.receipt_token_mint,
            &ctx.accounts.receipt_token_program,
            &mut ctx.accounts.fund_account,
            &mut ctx.accounts.reward_account,
            &ctx.accounts.user,
            &mut ctx.accounts.user_receipt_token_account,
            &mut ctx.accounts.user_fund_account,
            &mut ctx.accounts.user_reward_account,
        )?
        .process_withdraw_supported_token(
            &ctx.accounts.system_program,
            &ctx.accounts.supported_token_program,
            &ctx.accounts.supported_token_mint,
            &ctx.accounts.fund_supported_token_reserve_account,
            &ctx.accounts.user_supported_token_account,
            &mut ctx.accounts.fund_withdrawal_batch_account,
            &ctx.accounts.fund_reserve_account,
            &ctx.accounts.fund_treasury_account,
            request_id,
        )?);

        Ok(())
    }

    ////////////////////////////////////////////
    // UserFundWrappedTokenContext
    ////////////////////////////////////////////

    pub fn user_wrap_receipt_token(
        ctx: Context<UserFundWrappedTokenContext>,
        amount: u64,
    ) -> Result<()> {
        emit_cpi!(modules::fund::UserFundWrapService::new(
            &mut ctx.accounts.receipt_token_mint,
            &ctx.accounts.receipt_token_program,
            &mut ctx.accounts.wrapped_token_mint,
            &ctx.accounts.wrapped_token_program,
            &mut ctx.accounts.fund_account,
            &mut ctx.accounts.reward_account,
            &ctx.accounts.user,
            &mut ctx.accounts.user_receipt_token_account,
            &mut ctx.accounts.user_wrapped_token_account,
            &mut ctx.accounts.user_fund_account,
            &mut ctx.accounts.user_reward_account,
            &ctx.accounts.fund_wrap_account,
            &mut ctx.accounts.receipt_token_wrap_account,
            &mut ctx.accounts.fund_wrap_account_reward_account,
        )?
        .process_wrap_receipt_token(amount)?);

        Ok(())
    }

    pub fn user_wrap_receipt_token_if_needed(
        ctx: Context<UserFundWrappedTokenContext>,
        target_balance: u64,
    ) -> Result<()> {
        let event = modules::fund::UserFundWrapService::new(
            &mut ctx.accounts.receipt_token_mint,
            &ctx.accounts.receipt_token_program,
            &mut ctx.accounts.wrapped_token_mint,
            &ctx.accounts.wrapped_token_program,
            &mut ctx.accounts.fund_account,
            &mut ctx.accounts.reward_account,
            &ctx.accounts.user,
            &mut ctx.accounts.user_receipt_token_account,
            &mut ctx.accounts.user_wrapped_token_account,
            &mut ctx.accounts.user_fund_account,
            &mut ctx.accounts.user_reward_account,
            &ctx.accounts.fund_wrap_account,
            &mut ctx.accounts.receipt_token_wrap_account,
            &mut ctx.accounts.fund_wrap_account_reward_account,
        )?
        .process_wrap_receipt_token_if_needed(target_balance)?;

        if let Some(event) = event {
            emit_cpi!(event);
        }

        Ok(())
    }

    pub fn user_unwrap_receipt_token(
        ctx: Context<UserFundWrappedTokenContext>,
        amount: u64,
    ) -> Result<()> {
        emit_cpi!(modules::fund::UserFundWrapService::new(
            &mut ctx.accounts.receipt_token_mint,
            &ctx.accounts.receipt_token_program,
            &mut ctx.accounts.wrapped_token_mint,
            &ctx.accounts.wrapped_token_program,
            &mut ctx.accounts.fund_account,
            &mut ctx.accounts.reward_account,
            &ctx.accounts.user,
            &mut ctx.accounts.user_receipt_token_account,
            &mut ctx.accounts.user_wrapped_token_account,
            &mut ctx.accounts.user_fund_account,
            &mut ctx.accounts.user_reward_account,
            &ctx.accounts.fund_wrap_account,
            &mut ctx.accounts.receipt_token_wrap_account,
            &mut ctx.accounts.fund_wrap_account_reward_account,
        )?
        .process_unwrap_receipt_token(amount)?);

        Ok(())
    }

    ////////////////////////////////////////////
    // UserRewardAccountInitOrUpdateContext
    ////////////////////////////////////////////
    pub fn user_create_reward_account_idempotent(
        ctx: Context<UserRewardAccountInitOrUpdateContext>,
        desired_account_size: Option<u32>,
    ) -> Result<()> {
        let event = modules::reward::UserRewardConfigurationService::process_create_user_reward_account_idempotent(
            &ctx.accounts.system_program,
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.reward_account,
            &ctx.accounts.user,
            &ctx.accounts.user_receipt_token_account,
            &mut ctx.accounts.user_reward_account,
            ctx.bumps.user_reward_account,
            desired_account_size,
        )?;

        if let Some(event) = event {
            emit_cpi!(event);
        }

        Ok(())
    }

    pub fn user_update_reward_pools(ctx: Context<UserRewardContext>) -> Result<()> {
        emit_cpi!(modules::reward::UserRewardService::new(
            &ctx.accounts.receipt_token_mint,
            &ctx.accounts.user,
            &mut ctx.accounts.reward_account,
            &mut ctx.accounts.user_reward_account,
        )?
        .process_update_user_reward_pools()?);

        Ok(())
    }

    pub fn user_claim_reward(
        ctx: Context<UserRewardClaimContext>,
        is_bonus_pool: bool,
        reward_id: u16,
    ) -> Result<()> {
        emit_cpi!(modules::reward::UserRewardService::new(
            &ctx.accounts.receipt_token_mint,
            &ctx.accounts.user,
            &mut ctx.accounts.reward_account,
            &mut ctx.accounts.user_reward_account,
        )?
        .process_claim_user_reward(
            &ctx.accounts.reward_token_mint,
            &ctx.accounts.reward_token_program,
            &ctx.accounts.reward_reserve_account,
            &ctx.accounts.reward_token_reserve_account,
            &ctx.accounts.user_reward_token_account,
            is_bonus_pool,
            reward_id,
        )?);

        Ok(())
    }

    ////////////////////////////////////////////
    // UserReceiptTokenTransferContext
    ////////////////////////////////////////////

    #[instruction(discriminator = ExecuteInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn token_transfer_hook<'info>(
        ctx: Context<'_, '_, 'info, 'info, UserReceiptTokenTransferContext<'info>>,
        amount: u64,
    ) -> Result<()> {
        ctx.accounts.assert_is_transferring()?;

        let event = modules::fund::FundService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_transfer_hook(
            &mut ctx.accounts.reward_account,
            &mut ctx.accounts.source_receipt_token_account,
            &mut ctx.accounts.destination_receipt_token_account,
            &ctx.remaining_accounts,
            amount,
        )?;

        let event_authority_info = &ctx.remaining_accounts[4];
        let program_info = &ctx.remaining_accounts[5];
        events::emit_cpi(event_authority_info, program_info, &event)?;

        Ok(())
    }
}
