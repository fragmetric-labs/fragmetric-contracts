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
    use super::*;

    ////////////////////////////////////////////
    // AdminFundInitialContext
    ////////////////////////////////////////////

    pub fn admin_initialize_receipt_token_lock_authority(
        ctx: Context<AdminFundReceiptTokenLockAuthorityInitialContext>,
    ) -> Result<()> {
        modules::fund::process_initialize_receipt_token_lock_authority(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.receipt_token_lock_authority,
            ctx.bumps.receipt_token_lock_authority,
        )
    }

    pub fn admin_initialize_receipt_token_lock_account(
        _ctx: Context<AdminFundReceiptTokenLockAccountInitialContext>,
    ) -> Result<()> {
        Ok(())
    }

    pub fn admin_initialize_fund_account(
        ctx: Context<AdminFundAccountInitialContext>,
    ) -> Result<()> {
        modules::fund::process_initialize_fund_account(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
            ctx.bumps.fund_account,
        )
    }

    ////////////////////////////////////////////
    // AdminFundUpdateContext
    ////////////////////////////////////////////

    pub fn admin_update_fund_account_if_needed(
        ctx: Context<AdminFundAccountUpdateContext>,
    ) -> Result<()> {
        modules::fund::process_update_fund_account_if_needed(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
        )
    }

    ////////////////////////////////////////////
    // AdminReceiptTokenMintInitialContext
    ////////////////////////////////////////////

    pub fn admin_initialize_receipt_token_mint_authority(
        ctx: Context<AdminReceiptTokenMintAuthorityInitialContext>,
    ) -> Result<()> {
        modules::fund::process_initialize_receipt_token_mint_authority(
            &ctx.accounts.admin,
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.receipt_token_mint_authority,
            &ctx.accounts.receipt_token_program,
            ctx.bumps.receipt_token_mint_authority,
        )
    }

    #[interface(spl_transfer_hook_interface::initialize_extra_account_meta_list)]
    pub fn admin_initialize_extra_account_meta_list(
        ctx: Context<AdminReceiptTokenMintExtraAccountMetaListInitialContext>,
    ) -> Result<()> {
        modules::fund::process_initialize_extra_account_meta_list(
            ctx.accounts.extra_account_meta_list.as_ref(),
        )
    }

    ////////////////////////////////////////////
    // AdminReceiptTokenMintUpdateContext
    ////////////////////////////////////////////

    pub fn admin_update_extra_account_meta_list_if_needed(
        ctx: Context<AdminReceiptTokenMintExtraAccountMetaListUpdateContext>,
    ) -> Result<()> {
        modules::fund::process_update_extra_account_meta_list_if_needed(
            ctx.accounts.extra_account_meta_list.as_ref(),
        )
    }

    ////////////////////////////////////////////
    // AdminRewardInitialContext
    ////////////////////////////////////////////

    pub fn admin_initialize_reward_account(
        ctx: Context<AdminRewardAccountInitialContext>,
    ) -> Result<()> {
        modules::reward::process_initialize_reward_account(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.reward_account,
            ctx.bumps.reward_account,
        )
    }

    ////////////////////////////////////////////
    // AdminRewardUpdateContext
    ////////////////////////////////////////////

    pub fn admin_update_reward_accounts_if_needed(
        ctx: Context<AdminRewardAccountUpdateContext>,
        desired_account_size: Option<u32>,
        initialize: bool,
    ) -> Result<()> {
        modules::reward::process_update_reward_account_if_needed(
            &ctx.accounts.payer,
            &ctx.accounts.receipt_token_mint,
            &ctx.accounts.reward_account,
            &ctx.accounts.system_program,
            desired_account_size,
            initialize,
        )
    }

    ////////////////////////////////////////////
    // FundManagerFundContext
    ////////////////////////////////////////////

    pub fn fund_manager_update_sol_capacity_amount(
        ctx: Context<FundManagerFundContext>,
        capacity_amount: u64,
    ) -> Result<()> {
        modules::fund::process_update_sol_capacity_amount(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
            capacity_amount,
        )
    }

    pub fn fund_manager_update_supported_token_capacity_amount(
        ctx: Context<FundManagerFundContext>,
        token: Pubkey,
        capacity_amount: u64,
    ) -> Result<()> {
        modules::fund::process_update_supported_token_capacity_amount(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
            token,
            capacity_amount,
        )
    }

    pub fn fund_manager_update_withdrawal_enabled_flag(
        ctx: Context<FundManagerFundContext>,
        enabled: bool,
    ) -> Result<()> {
        modules::fund::process_update_withdrawal_enabled_flag(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
            enabled,
        )
    }

    pub fn fund_manager_update_sol_withdrawal_fee_rate(
        ctx: Context<FundManagerFundContext>,
        sol_withdrawal_fee_rate: u16,
    ) -> Result<()> {
        modules::fund::process_update_sol_withdrawal_fee_rate(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
            sol_withdrawal_fee_rate,
        )
    }

    pub fn fund_manager_update_batch_processing_threshold(
        ctx: Context<FundManagerFundContext>,
        amount: Option<u64>,
        duration: Option<i64>,
    ) -> Result<()> {
        modules::fund::process_update_batch_processing_threshold(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
            amount,
            duration,
        )
    }

    ////////////////////////////////////////////
    // FundManagerFundSupportedTokenInitialContext
    ////////////////////////////////////////////

    pub fn fund_manager_initialize_supported_token_authority(
        ctx: Context<FundManagerFundSupportedTokenAuthorityInitialContext>,
    ) -> Result<()> {
        modules::fund::process_initialize_supported_token_authority(
            &ctx.accounts.receipt_token_mint,
            &ctx.accounts.supported_token_mint,
            &mut ctx.accounts.supported_token_authority,
            ctx.bumps.supported_token_authority,
        )
    }

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
        modules::fund::process_add_supported_token(
            &ctx.accounts.receipt_token_mint,
            &ctx.accounts.supported_token_mint,
            &mut ctx.accounts.fund_account,
            &ctx.accounts.supported_token_program,
            capacity_amount,
            pricing_source,
            ctx.remaining_accounts,
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
        modules::reward::process_add_reward_pool_holder(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.reward_account,
            name,
            description,
            pubkeys,
        )
    }

    pub fn fund_manager_add_reward_pool(
        ctx: Context<FundManagerRewardContext>,
        name: String,
        holder_id: Option<u8>,
        custom_contribution_accrual_rate_enabled: bool,
    ) -> Result<()> {
        modules::reward::process_add_reward_pool(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.reward_account,
            name,
            holder_id,
            custom_contribution_accrual_rate_enabled,
            Clock::get()?.slot,
        )
    }

    pub fn fund_manager_close_reward_pool(
        ctx: Context<FundManagerRewardContext>,
        reward_pool_id: u8,
    ) -> Result<()> {
        modules::reward::process_close_reward_pool(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.reward_account,
            reward_pool_id,
            Clock::get()?.slot,
        )
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
        modules::reward::process_add_reward(
            &ctx.accounts.receipt_token_mint,
            ctx.accounts.reward_token_mint.as_deref(),
            &mut ctx.accounts.reward_account,
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
        modules::reward::process_settle_reward(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.reward_account,
            reward_pool_id,
            reward_id,
            amount,
            Clock::get()?.slot,
        )
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

    pub fn operator_process_fund_withdrawal_job<'info>(
        ctx: Context<'_, '_, 'info, 'info, OperatorFundContext<'info>>,
        forced: bool,
    ) -> Result<()> {
        modules::operator::process_process_fund_withdrawal_job(
            &ctx.accounts.operator,
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.receipt_token_lock_account,
            &ctx.accounts.receipt_token_lock_authority,
            &mut ctx.accounts.fund_account,
            &ctx.accounts.receipt_token_program,
            ctx.remaining_accounts,
            forced,
            Clock::get()?.unix_timestamp,
        )
    }

    pub fn operator_update_prices<'info>(
        ctx: Context<'_, '_, 'info, 'info, OperatorFundContext<'info>>,
    ) -> Result<()> {
        modules::fund::process_update_prices(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
            ctx.remaining_accounts,
        )
    }

    ////////////////////////////////////////////
    // OperatorRewardContext
    ////////////////////////////////////////////

    pub fn operator_update_reward_pools(ctx: Context<OperatorRewardContext>) -> Result<()> {
        modules::reward::process_update_reward_pools(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.reward_account,
            Clock::get()?.slot,
        )
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
        modules::fund::process_initialize_user_fund_account(
            &ctx.accounts.user,
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.user_fund_account,
            ctx.bumps.user_fund_account,
        )
    }

    ////////////////////////////////////////////
    // UserFundUpdateContext
    ////////////////////////////////////////////

    pub fn user_update_fund_account_if_needed(
        ctx: Context<UserFundAccountUpdateContext>,
    ) -> Result<()> {
        modules::fund::process_update_user_fund_account_if_needed(
            &ctx.accounts.user,
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.user_fund_account,
        )
    }

    ////////////////////////////////////////////
    // UserFundContext
    ////////////////////////////////////////////

    pub fn user_deposit_sol<'info>(
        ctx: Context<'_, '_, 'info, 'info, UserFundContext<'info>>,
        amount: u64,
        metadata: Option<modules::fund::DepositMetadata>,
    ) -> Result<()> {
        ctx.accounts.check_user_sol_balance(amount)?;
        modules::fund::process_deposit_sol(
            &ctx.accounts.user,
            &mut ctx.accounts.receipt_token_mint,
            &ctx.accounts.receipt_token_mint_authority,
            &mut ctx.accounts.user_receipt_token_account,
            &mut ctx.accounts.fund_account,
            &mut ctx.accounts.reward_account,
            &mut ctx.accounts.user_fund_account,
            &mut ctx.accounts.user_reward_account,
            &ctx.accounts.system_program,
            &ctx.accounts.receipt_token_program,
            &ctx.accounts.instructions_sysvar,
            Clock::get()?,
            ctx.remaining_accounts,
            amount,
            metadata,
        )
    }

    pub fn user_request_withdrawal(
        ctx: Context<UserFundContext>,
        receipt_token_amount: u64,
    ) -> Result<()> {
        ctx.accounts
            .check_user_receipt_token_balance(receipt_token_amount)?;
        modules::fund::process_request_withdrawal(
            &ctx.accounts.user,
            &mut ctx.accounts.receipt_token_mint,
            &ctx.accounts.receipt_token_mint_authority,
            &mut ctx.accounts.receipt_token_lock_account,
            &mut ctx.accounts.user_receipt_token_account,
            &mut ctx.accounts.fund_account,
            &mut ctx.accounts.reward_account,
            &mut ctx.accounts.user_fund_account,
            &mut ctx.accounts.user_reward_account,
            &ctx.accounts.receipt_token_program,
            Clock::get()?,
            receipt_token_amount,
        )
    }

    pub fn user_cancel_withdrawal_request(
        ctx: Context<UserFundContext>,
        request_id: u64,
    ) -> Result<()> {
        modules::fund::process_cancel_withdrawal_request(
            &ctx.accounts.user,
            &mut ctx.accounts.receipt_token_mint,
            &ctx.accounts.receipt_token_mint_authority,
            &mut ctx.accounts.receipt_token_lock_account,
            &ctx.accounts.receipt_token_lock_authority,
            &mut ctx.accounts.user_receipt_token_account,
            &mut ctx.accounts.fund_account,
            &mut ctx.accounts.reward_account,
            &mut ctx.accounts.user_fund_account,
            &mut ctx.accounts.user_reward_account,
            &ctx.accounts.receipt_token_program,
            request_id,
            Clock::get()?.slot,
        )
    }

    pub fn user_withdraw(ctx: Context<UserFundContext>, request_id: u64) -> Result<()> {
        modules::fund::process_withdraw(
            &ctx.accounts.user,
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.fund_account,
            &mut ctx.accounts.user_fund_account,
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
        ctx.accounts.check_user_supported_token_balance(amount)?;
        modules::fund::process_deposit_supported_token(
            &ctx.accounts.user,
            &mut ctx.accounts.receipt_token_mint,
            &ctx.accounts.receipt_token_mint_authority,
            &mut ctx.accounts.user_receipt_token_account,
            &ctx.accounts.supported_token_mint,
            &ctx.accounts.supported_token_account,
            &ctx.accounts.user_supported_token_account,
            &mut ctx.accounts.fund_account,
            &mut ctx.accounts.reward_account,
            &mut ctx.accounts.user_fund_account,
            &mut ctx.accounts.user_reward_account,
            &ctx.accounts.receipt_token_program,
            &ctx.accounts.supported_token_program,
            &ctx.accounts.instruction_sysvar,
            Clock::get()?,
            ctx.remaining_accounts,
            amount,
            metadata,
        )
    }

    ////////////////////////////////////////////
    // UserRewardInitialContext
    ////////////////////////////////////////////

    pub fn user_initialize_reward_account(
        ctx: Context<UserRewardAccountInitialContext>,
    ) -> Result<()> {
        modules::reward::process_initialize_user_reward_account(
            &ctx.accounts.user,
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.user_reward_account,
            ctx.bumps.user_reward_account,
        )
    }

    ////////////////////////////////////////////
    // UserRewardContext
    ////////////////////////////////////////////

    pub fn user_update_reward_accounts_if_needed(
        ctx: Context<UserRewardContext>,
        desired_account_size: Option<u32>,
        initialize: bool,
    ) -> Result<()> {
        modules::reward::process_update_user_reward_account_if_needed(
            &ctx.accounts.user,
            &ctx.accounts.receipt_token_mint,
            &ctx.accounts.user_reward_account,
            &ctx.accounts.system_program,
            desired_account_size,
            initialize,
        )
    }

    pub fn user_update_reward_pools(ctx: Context<UserRewardContext>) -> Result<()> {
        ctx.accounts.check_user_reward_account_constraint()?;
        modules::reward::process_update_user_reward_pools(
            &mut ctx.accounts.reward_account,
            &mut ctx.accounts.user_reward_account,
            Clock::get()?.slot,
        )
    }

    #[allow(unused_variables)]
    pub fn user_claim_rewards(
        ctx: Context<UserRewardContext>,
        reward_pool_id: u8,
        reward_id: u8,
    ) -> Result<()> {
        ctx.accounts.check_user_reward_account_constraint()?;
        modules::reward::process_claim_rewards()
    }

    ////////////////////////////////////////////
    // UserReceiptTokenTransferContext
    ////////////////////////////////////////////

    #[interface(spl_transfer_hook_interface::execute)]
    pub fn token_transfer_hook(
        ctx: Context<UserReceiptTokenTransferContext>,
        amount: u64,
    ) -> Result<()> {
        UserReceiptTokenTransferContext::handle_transfer(ctx, amount)
    }
}
