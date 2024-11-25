use anchor_lang::prelude::*;

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
    use anchor_spl::token_interface::{TokenAccount, TokenInterface};
    use super::*;
    use crate::modules::normalization::NormalizedTokenPoolAccount;
    use crate::utils::AccountInfoExt;

    ////////////////////////////////////////////
    // AdminFundInitialContext
    ////////////////////////////////////////////

    pub fn admin_initialize_receipt_token_lock_account(
        _ctx: Context<AdminFundReceiptTokenLockAccountInitialContext>,
    ) -> Result<()> {
        Ok(())
    }

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
            ctx.bumps.fund_account,
        )
    }

    pub fn admin_initialize_fund_normalized_token_account<'info>(
        ctx: Context<'_, '_, 'info, 'info, AdminFundNormalizedTokenAccountInitialContext<'info>>,
    ) -> Result<()> {
        modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_set_normalized_token(
            &ctx.accounts.fund_normalized_token_account,
            &mut ctx.accounts.normalized_token_mint,
            &ctx.accounts.normalized_token_program,
            &mut ctx.accounts.normalized_token_pool_account,
            ctx.remaining_accounts,
        )
    }

    pub fn admin_initialize_jito_restaking_vault_receipt_token_account<'info>(
        ctx: Context<'_, '_, 'info, 'info, AdminFundJitoRestakingProtocolAccountInitialContext<'info>>,
    ) -> Result<()> {
        modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
            .process_add_restaking_vault(
                &ctx.accounts.fund_vault_supported_token_account,
                &ctx.accounts.fund_vault_receipt_token_account,

                &ctx.accounts.vault_supported_token_mint,
                &ctx.accounts.vault_supported_token_program,

                &ctx.accounts.vault,
                &ctx.accounts.vault_program,
                &ctx.accounts.vault_receipt_token_mint,
                &ctx.accounts.vault_receipt_token_program,
                
                ctx.remaining_accounts,
            )
    }

    ////////////////////////////////////////////
    // AdminFundUpdateContext
    ////////////////////////////////////////////

    pub fn admin_update_fund_account_if_needed(
        ctx: Context<AdminFundAccountUpdateContext>,
    ) -> Result<()> {
        modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_update_fund_account_if_needed()
    }

    ////////////////////////////////////////////
    // AdminNormalizedTokenPoolInitialContext
    ////////////////////////////////////////////

    pub fn admin_initialize_normalized_token_pool_account(
        ctx: Context<AdminNormalizedTokenPoolInitialContext>,
    ) -> Result<()> {
        modules::normalization::NormalizedTokenPoolConfigurationService::new(
            &mut ctx.accounts.normalized_token_pool_account,
            &ctx.accounts.normalized_token_mint,
            &ctx.accounts.normalized_token_program,
        )?
        .process_initialize_normalized_token_pool_account(
            &ctx.accounts.admin,
            ctx.bumps.normalized_token_pool_account,
        )
    }

    pub fn admin_update_normalized_token_pool_account_if_needed(
        ctx: Context<AdminNormalizedTokenPoolUpdateContext>,
    ) -> Result<()> {
        modules::normalization::NormalizedTokenPoolConfigurationService::new(
            &mut ctx.accounts.normalized_token_pool_account,
            &ctx.accounts.normalized_token_mint,
            &ctx.accounts.normalized_token_program,
        )?
        .process_update_normalized_token_pool_account_if_needed()
    }

    ////////////////////////////////////////////
    // AdminReceiptTokenMintInitialContext
    ////////////////////////////////////////////

    #[interface(spl_transfer_hook_interface::initialize_extra_account_meta_list)]
    pub fn admin_initialize_extra_account_meta_list(
        ctx: Context<AdminReceiptTokenMintExtraAccountMetaListInitialContext>,
    ) -> Result<()> {
        modules::fund::FundReceiptTokenConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &ctx.accounts.extra_account_meta_list,
        )?
        .process_initialize_extra_account_meta_list()
    }

    ////////////////////////////////////////////
    // AdminReceiptTokenMintUpdateContext
    ////////////////////////////////////////////

    pub fn admin_update_extra_account_meta_list_if_needed(
        ctx: Context<AdminReceiptTokenMintExtraAccountMetaListUpdateContext>,
    ) -> Result<()> {
        modules::fund::FundReceiptTokenConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &ctx.accounts.extra_account_meta_list,
        )?
        .process_update_extra_account_meta_list_if_needed()
    }

    ////////////////////////////////////////////
    // AdminRewardInitialContext
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
    // AdminRewardUpdateContext
    ////////////////////////////////////////////

    pub fn admin_update_reward_accounts_if_needed(
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
    // FundManagerFundContext
    ////////////////////////////////////////////

    pub fn fund_manager_update_sol_capacity_amount(
        ctx: Context<FundManagerFundContext>,
        capacity_amount: u64,
    ) -> Result<()> {
        modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_update_sol_capacity_amount(capacity_amount)
    }

    pub fn fund_manager_update_supported_token_capacity_amount(
        ctx: Context<FundManagerFundContext>,
        token_mint: Pubkey,
        capacity_amount: u64,
    ) -> Result<()> {
        modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_update_supported_token_capacity_amount(&token_mint, capacity_amount)
    }

    pub fn fund_manager_update_withdrawal_enabled_flag(
        ctx: Context<FundManagerFundContext>,
        enabled: bool,
    ) -> Result<()> {
        modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_update_withdrawal_enabled_flag(enabled)
    }

    pub fn fund_manager_update_sol_withdrawal_fee_rate(
        ctx: Context<FundManagerFundContext>,
        sol_withdrawal_fee_rate: u16,
    ) -> Result<()> {
        modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_update_sol_withdrawal_fee_rate(sol_withdrawal_fee_rate)
    }

    pub fn fund_manager_update_batch_processing_threshold(
        ctx: Context<FundManagerFundContext>,
        amount: Option<u64>,
        duration: Option<i64>,
    ) -> Result<()> {
        modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_update_batch_processing_threshold(amount, duration)
    }

    ////////////////////////////////////////////
    // FundManagerFundSupportedTokenInitialContext
    ////////////////////////////////////////////

    pub fn fund_manager_initialize_supported_token_account(
        _ctx: Context<FundManagerFundSupportedTokenAccountInitialContext>,
    ) -> Result<()> {
        Ok(())
    }

    ////////////////////////////////////////////
    // FundManagerFundSupportedTokenContext
    ////////////////////////////////////////////

    pub fn fund_manager_add_supported_token<'info>(
        ctx: Context<'_, '_, 'info, 'info, FundManagerFundSupportedTokenContext<'info>>,
        capacity_amount: u64,
        pricing_source: modules::pricing::TokenPricingSource,
    ) -> Result<()> {
        modules::fund::FundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_add_supported_token(
            &ctx.accounts.supported_token_account,
            &ctx.accounts.supported_token_mint,
            &ctx.accounts.supported_token_program,
            capacity_amount,
            pricing_source,
            ctx.remaining_accounts,
        )
    }

    ////////////////////////////////////////////
    // FundManagerNormalizedTokenPoolSupportedTokenInitialContext
    ////////////////////////////////////////////

    pub fn fund_manager_initialize_supported_token_lock_account(
        _ctx: Context<FundManagerNormalizedTokenPoolSupportedTokenLockAccountInitialContext>,
    ) -> Result<()> {
        Ok(())
    }

    ////////////////////////////////////////////
    // FundManagerNormalizedTokenPoolSupportedTokenContext
    ////////////////////////////////////////////

    pub fn fund_manager_add_normalized_token_pool_supported_token(
        ctx: Context<FundManagerNormalizedTokenPoolSupportedTokenContext>,
    ) -> Result<()> {
        modules::normalization::NormalizedTokenPoolConfigurationService::new(
            &mut ctx.accounts.normalized_token_pool_account,
            &ctx.accounts.normalized_token_mint,
            &ctx.accounts.normalized_token_program,
        )?
        .process_add_supported_token(
            &ctx.accounts.supported_token_lock_account,
            &ctx.accounts.supported_token_mint,
            &ctx.accounts.supported_token_program,
        )
    }

    ////////////////////////////////////////////
    // FundManagerRewardContext
    ////////////////////////////////////////////

    pub fn fund_manager_add_reward_pool_holder(
        ctx: Context<FundManagerRewardContext>,
        name: String,
        description: String,
        pubkeys: Vec<Pubkey>,
    ) -> Result<()> {
        modules::reward::RewardConfigurationService::new(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.reward_account,
        )?
        .process_add_reward_pool_holder(name, description, pubkeys)
    }

    pub fn fund_manager_add_reward_pool(
        ctx: Context<FundManagerRewardContext>,
        name: String,
        holder_id: Option<u8>,
        custom_contribution_accrual_rate_enabled: bool,
    ) -> Result<()> {
        modules::reward::RewardConfigurationService::new(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.reward_account,
        )?
        .process_add_reward_pool(
            name,
            holder_id,
            custom_contribution_accrual_rate_enabled,
        )
    }

    pub fn fund_manager_close_reward_pool(
        ctx: Context<FundManagerRewardContext>,
        reward_pool_id: u8,
    ) -> Result<()> {
        modules::reward::RewardConfigurationService::new(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.reward_account,
        )?
        .process_close_reward_pool(reward_pool_id)
    }

    ////////////////////////////////////////////
    // FundManagerRewardDistributionContext
    ////////////////////////////////////////////

    pub fn fund_manager_add_reward(
        ctx: Context<FundManagerRewardDistributionContext>,
        name: String,
        description: String,
        reward_type: modules::reward::RewardType,
    ) -> Result<()> {
        modules::reward::RewardConfigurationService::new(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.reward_account,
        )?
        .process_add_reward(
            ctx.accounts.reward_token_mint.as_deref(),
            ctx.accounts.reward_token_program.as_ref(),
            name,
            description,
            reward_type,
        )
    }

    pub fn fund_manager_settle_reward(
        ctx: Context<FundManagerRewardDistributionContext>,
        reward_pool_id: u8,
        reward_id: u16,
        amount: u64,
    ) -> Result<()> {
        modules::reward::RewardConfigurationService::new(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.reward_account,
        )?
        .process_settle_reward(reward_pool_id, reward_id, amount)
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

    pub fn operator_run<'info>(
        ctx: Context<'_, '_, 'info, 'info, OperatorFundContext<'info>>,
        force_reset_command: Option<modules::fund::command::OperationCommandEntry>,
    ) -> Result<()> {
        if force_reset_command.is_some() {
            require_eq!(ctx.accounts.operator.key(), FUND_MANAGER_PUBKEY);
        }

        modules::fund::FundService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_run(ctx.remaining_accounts, force_reset_command)
    }

    // TODO v0.3/operation: deprecate old run
    pub fn operator_deprecating_run<'info>(
        ctx: Context<'_, '_, 'info, 'info, OperatorFundContext<'info>>,
        command: u8,
    ) -> Result<()> {
        let clock = Clock::get()?;
        modules::operation::process_run(
            &ctx.accounts.operator,
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
            ctx.remaining_accounts,
            clock.unix_timestamp,
            clock.slot,
            command,
        )
    }

    ////////////////////////////////////////////
    // OperatorRewardContext
    ////////////////////////////////////////////

    pub fn operator_update_reward_pools(ctx: Context<OperatorRewardContext>) -> Result<()> {
        modules::reward::RewardService::new(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.reward_account,
        )?
        .process_update_reward_pools()
    }

    ////////////////////////////////////////////
    // UserFundInitialContext
    ////////////////////////////////////////////

    pub fn user_initialize_receipt_token_account(
        _ctx: Context<UserFundReceiptTokenAccountInitialContext>,
    ) -> Result<()> {
        Ok(())
    }

    pub fn user_initialize_fund_account(ctx: Context<UserFundAccountInitialContext>) -> Result<()> {
        modules::fund::UserFundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &ctx.accounts.user,
            &mut ctx.accounts.user_fund_account,
        )?
        .process_initialize_user_fund_account(ctx.bumps.user_fund_account)
    }

    ////////////////////////////////////////////
    // UserFundUpdateContext
    ////////////////////////////////////////////

    pub fn user_update_fund_account_if_needed(
        ctx: Context<UserFundAccountUpdateContext>,
    ) -> Result<()> {
        modules::fund::UserFundConfigurationService::new(
            &mut ctx.accounts.receipt_token_mint,
            &ctx.accounts.user,
            &mut ctx.accounts.user_fund_account,
        )?
        .process_update_user_fund_account_if_needed()
    }

    ////////////////////////////////////////////
    // UserFundContext
    ////////////////////////////////////////////

    pub fn user_deposit_sol<'info>(
        ctx: Context<'_, '_, 'info, 'info, UserFundContext<'info>>,
        amount: u64,
        metadata: Option<modules::fund::DepositMetadata>,
    ) -> Result<()> {
        modules::fund::UserFundService::new(
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
            &ctx.accounts.fund_reserve_account,
            &ctx.accounts.system_program,
            &ctx.accounts.instructions_sysvar,
            ctx.remaining_accounts,
            amount,
            metadata,
            &ADMIN_PUBKEY,
        )
    }

    pub fn user_request_withdrawal(
        ctx: Context<UserFundContext>,
        receipt_token_amount: u64,
    ) -> Result<()> {
        modules::fund::UserFundService::new(
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
            receipt_token_amount,
        )
    }

    pub fn user_cancel_withdrawal_request(
        ctx: Context<UserFundContext>,
        request_id: u64,
    ) -> Result<()> {
        modules::fund::UserFundService::new(
            &mut ctx.accounts.receipt_token_mint,
            &ctx.accounts.receipt_token_program,
            &mut ctx.accounts.fund_account,
            &mut ctx.accounts.reward_account,
            &ctx.accounts.user,
            &mut ctx.accounts.user_receipt_token_account,
            &mut ctx.accounts.user_fund_account,
            &mut ctx.accounts.user_reward_account,
        )?
        .process_cancel_withdrawal_request(&mut ctx.accounts.receipt_token_lock_account, request_id)
    }

    pub fn user_withdraw(ctx: Context<UserFundContext>, request_id: u64) -> Result<()> {
        modules::fund::UserFundService::new(
            &mut ctx.accounts.receipt_token_mint,
            &ctx.accounts.receipt_token_program,
            &mut ctx.accounts.fund_account,
            &mut ctx.accounts.reward_account,
            &ctx.accounts.user,
            &mut ctx.accounts.user_receipt_token_account,
            &mut ctx.accounts.user_fund_account,
            &mut ctx.accounts.user_reward_account,
        )?
        .process_withdraw(
            &ctx.accounts.fund_reserve_account,
            ctx.bumps.fund_reserve_account,
            &ctx.accounts.fund_treasury_account,
            &ctx.accounts.system_program,
            request_id,
        )
    }

    ////////////////////////////////////////////
    // UserFundSupportedTokenContext
    ////////////////////////////////////////////

    pub fn user_deposit_supported_token<'info>(
        ctx: Context<'_, '_, 'info, 'info, UserFundSupportedTokenContext<'info>>,
        amount: u64,
        metadata: Option<modules::fund::DepositMetadata>,
    ) -> Result<()> {
        modules::fund::UserFundService::new(
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
            &ctx.accounts.supported_token_mint,
            &ctx.accounts.fund_supported_token_account,
            &ctx.accounts.user_supported_token_account,
            &ctx.accounts.supported_token_program,
            &ctx.accounts.instructions_sysvar,
            ctx.remaining_accounts,
            amount,
            metadata,
            &ADMIN_PUBKEY,
        )
    }

    ////////////////////////////////////////////
    // UserRewardInitialContext
    ////////////////////////////////////////////

    pub fn user_initialize_reward_account(
        ctx: Context<UserRewardAccountInitialContext>,
    ) -> Result<()> {
        modules::reward::UserRewardConfigurationService::new(
            &ctx.accounts.receipt_token_mint,
            &ctx.accounts.user,
            &mut ctx.accounts.user_reward_account,
        )?
        .process_initialize_user_reward_account(ctx.bumps.user_reward_account)
    }

    ////////////////////////////////////////////
    // UserRewardContext
    ////////////////////////////////////////////

    #[allow(unused_variables)]
    pub fn user_update_reward_accounts_if_needed(
        ctx: Context<UserRewardAccountUpdateContext>,
        desired_account_size: Option<u32>,
        initialize: bool, // deprecated
    ) -> Result<()> {
        modules::reward::UserRewardConfigurationService::new(
            &ctx.accounts.receipt_token_mint,
            &ctx.accounts.user,
            &mut ctx.accounts.user_reward_account,
        )?
        .process_update_user_reward_account_if_needed(
            &ctx.accounts.system_program,
            desired_account_size,
        )
    }

    pub fn user_update_reward_pools(ctx: Context<UserRewardContext>) -> Result<()> {
        modules::reward::UserRewardService::new(
            &mut ctx.accounts.reward_account,
            &mut ctx.accounts.user_reward_account,
        )?
        .process_update_user_reward_pools()
    }

    #[allow(unused_variables)]
    pub fn user_claim_rewards(
        ctx: Context<UserRewardContext>,
        reward_pool_id: u8,
        reward_id: u8,
    ) -> Result<()> {
        modules::reward::UserRewardService::new(
            &mut ctx.accounts.reward_account,
            &mut ctx.accounts.user_reward_account,
        )?
        .process_claim_user_rewards()
    }

    ////////////////////////////////////////////
    // UserReceiptTokenTransferContext
    ////////////////////////////////////////////

    #[interface(spl_transfer_hook_interface::execute)]
    pub fn token_transfer_hook<'info>(
        ctx: Context<'_, '_, 'info, 'info, UserReceiptTokenTransferContext<'info>>,
        amount: u64,
    ) -> Result<()> {
        ctx.accounts.assert_is_transferring()?;

        modules::fund::FundService::new(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )?
        .process_transfer_hook(
            &mut ctx.accounts.reward_account,
            &mut ctx.accounts.source_receipt_token_account,
            &mut ctx.accounts.destination_receipt_token_account,
            ctx.remaining_accounts,
            amount,
        )
    }
}
