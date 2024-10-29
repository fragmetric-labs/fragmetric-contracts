use anchor_lang::prelude::*;

pub(crate) mod constants;
pub(crate) mod errors;
pub(crate) mod events;
pub(crate) mod modules;
pub(crate) mod utils;

mod instructions;

use constants::*;
use instructions::*;
use modules::restaking::jito::*;

#[program]
pub mod restaking {
    use super::*;


    ////////////////////////////////////////////
    // AdminFundInitialContext
    ////////////////////////////////////////////

    pub fn admin_initialize_receipt_token_lock_authority(
        ctx: Context<AdminFundReceiptTokenLockAuthorityInitialContext>,
    ) -> Result<()> {
        AdminFundReceiptTokenLockAuthorityInitialContext::initialize_receipt_token_lock_authority(
            ctx,
        )
    }

    pub fn admin_initialize_receipt_token_lock_account(
        ctx: Context<AdminFundReceiptTokenLockAccountInitialContext>,
    ) -> Result<()> {
        AdminFundReceiptTokenLockAccountInitialContext::initialize_receipt_token_lock_account(ctx)
    }

    pub fn admin_initialize_fund_account(
        ctx: Context<AdminFundAccountInitialContext>,
    ) -> Result<()> {
        AdminFundAccountInitialContext::initialize_fund_account(ctx)
    }

    ////////////////////////////////////////////
    // AdminFundContext
    ////////////////////////////////////////////

    pub fn admin_update_fund_account(ctx: Context<AdminFundContext>) -> Result<()> {
        AdminFundContext::update_fund_account(ctx)
    }

    ////////////////////////////////////////////
    // AdminReceiptTokenMintInitialContext
    ////////////////////////////////////////////

    pub fn admin_initialize_receipt_token_mint_authority(
        ctx: Context<AdminReceiptTokenMintAuthorityInitialContext>,
    ) -> Result<()> {
        AdminReceiptTokenMintAuthorityInitialContext::initialize_mint_authority(ctx)
    }

    #[interface(spl_transfer_hook_interface::initialize_extra_account_meta_list)]
    pub fn admin_initialize_receipt_token_mint_extra_account_meta_list(
        ctx: Context<AdminReceiptTokenMintExtraAccountMetaListInitialContext>,
    ) -> Result<()> {
        AdminReceiptTokenMintExtraAccountMetaListInitialContext::initialize_extra_account_meta_list(
            ctx,
        )
    }

    ////////////////////////////////////////////
    // AdminReceiptTokenMintContext
    ////////////////////////////////////////////

    pub fn admin_update_receipt_token_mint_extra_account_meta_list(
        ctx: Context<AdminReceiptTokenMintContext>,
    ) -> Result<()> {
        AdminReceiptTokenMintContext::update_extra_account_meta_list(ctx)
    }

    ////////////////////////////////////////////
    // AdminRewardInitialContext
    ////////////////////////////////////////////

    pub fn admin_initialize_reward_account(
        ctx: Context<AdminRewardAccountInitialContext>,
    ) -> Result<()> {
        AdminRewardAccountInitialContext::initialize_reward_account(ctx)
    }

    ////////////////////////////////////////////
    // AdminRewardContext
    ////////////////////////////////////////////

    pub fn admin_update_reward_accounts_if_needed(
        ctx: Context<AdminRewardContext>,
        desired_account_size: Option<u32>,
        initialize: bool,
    ) -> Result<()> {
        AdminRewardContext::update_accounts_if_needed(ctx, desired_account_size, initialize)
    }

    ////////////////////////////////////////////
    // FundManagerFundContext
    ////////////////////////////////////////////

    pub fn fund_manager_update_sol_capacity_amount(
        ctx: Context<FundManagerFundContext>,
        capacity_amount: u64,
    ) -> Result<()> {
        FundManagerFundContext::update_sol_capacity_amount(ctx, capacity_amount)
    }

    pub fn fund_manager_update_supported_token_capacity_amount(
        ctx: Context<FundManagerFundContext>,
        token: Pubkey,
        capacity_amount: u64,
    ) -> Result<()> {
        FundManagerFundContext::update_supported_token_capacity_amount(ctx, token, capacity_amount)
    }

    pub fn fund_manager_update_withdrawal_enabled_flag(
        ctx: Context<FundManagerFundContext>,
        enabled: bool,
    ) -> Result<()> {
        FundManagerFundContext::update_withdrawal_enabled_flag(ctx, enabled)
    }

    pub fn fund_manager_update_sol_withdrawal_fee_rate(
        ctx: Context<FundManagerFundContext>,
        sol_withdrawal_fee_rate: u16,
    ) -> Result<()> {
        FundManagerFundContext::update_sol_withdrawal_fee_rate(ctx, sol_withdrawal_fee_rate)
    }

    pub fn fund_manager_update_batch_processing_threshold(
        ctx: Context<FundManagerFundContext>,
        amount: Option<u64>,
        duration: Option<i64>,
    ) -> Result<()> {
        FundManagerFundContext::update_batch_processing_threshold(ctx, amount, duration)
    }

    ////////////////////////////////////////////
    // FundManagerFundSupportedTokenInitialContext
    ////////////////////////////////////////////

    pub fn fund_manager_initialize_supported_token_authority(
        ctx: Context<FundManagerFundSupportedTokenAuthorityInitialContext>,
    ) -> Result<()> {
        FundManagerFundSupportedTokenAuthorityInitialContext::initialize_supported_token_authority(
            ctx,
        )
    }

    pub fn fund_manager_initialize_supported_token_account(
        ctx: Context<FundManagerFundSupportedTokenAccountInitialContext>,
    ) -> Result<()> {
        FundManagerFundSupportedTokenAccountInitialContext::intialize_supported_token_account(ctx)
    }

    ////////////////////////////////////////////
    // FundManagerFundSupportedTokenContext
    ////////////////////////////////////////////

    pub fn fund_manager_add_supported_token(
        ctx: Context<FundManagerFundSupportedTokenContext>,
        capacity_amount: u64,
        pricing_source: modules::fund::TokenPricingSource,
    ) -> Result<()> {
        FundManagerFundSupportedTokenContext::add_supported_token(
            ctx,
            capacity_amount,
            pricing_source,
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
        FundManagerRewardContext::add_reward_pool_holder(ctx, name, description, pubkeys)
    }

    pub fn fund_manager_add_reward_pool(
        ctx: Context<FundManagerRewardContext>,
        name: String,
        holder_id: Option<u8>,
        custom_contribution_accrual_rate_enabled: bool,
    ) -> Result<()> {
        FundManagerRewardContext::add_reward_pool(
            ctx,
            name,
            holder_id,
            custom_contribution_accrual_rate_enabled,
        )
    }

    pub fn fund_manager_close_reward_pool(
        ctx: Context<FundManagerRewardContext>,
        reward_pool_id: u8,
    ) -> Result<()> {
        FundManagerRewardContext::close_reward_pool(ctx, reward_pool_id)
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
        FundManagerRewardDistributionContext::add_reward(ctx, name, description, reward_type)
    }

    pub fn fund_manager_settle_reward(
        ctx: Context<FundManagerRewardDistributionContext>,
        reward_pool_id: u8,
        reward_id: u16,
        amount: u64,
    ) -> Result<()> {
        FundManagerRewardDistributionContext::settle_reward(ctx, reward_pool_id, reward_id, amount)
    }

    ////////////////////////////////////////////
    // OperatorEmptyContext
    ////////////////////////////////////////////

    pub fn operator_log_message(ctx: Context<OperatorEmptyContext>, message: String) -> Result<()> {
        OperatorEmptyContext::log_message(ctx, message)
    }

    ////////////////////////////////////////////
    // OperatorFundContext
    ////////////////////////////////////////////

    pub fn operator_process_fund_withdrawal_job<'info>(
        ctx: Context<'_, '_, '_, 'info, OperatorFundContext<'info>>,
        forced: bool,
    ) -> Result<()> {
        OperatorFundContext::process_fund_withdrawal_job(ctx, forced)
    }

    pub fn operator_update_prices(ctx: Context<OperatorFundContext>) -> Result<()> {
        OperatorFundContext::update_prices(ctx)
    }

    ////////////////////////////////////////////
    // OperatorRewardContext
    ////////////////////////////////////////////

    pub fn operator_update_reward_pools(ctx: Context<OperatorRewardContext>) -> Result<()> {
        OperatorRewardContext::update_reward_pools(ctx)
    }

    ////////////////////////////////////////////
    // UserFundInitialContext
    ////////////////////////////////////////////

    pub fn user_initialize_receipt_token_account(
        ctx: Context<UserFundReceiptTokenAccountInitialContext>,
    ) -> Result<()> {
        UserFundReceiptTokenAccountInitialContext::initialize_receipt_token_account(ctx)
    }

    pub fn user_initialize_fund_account(ctx: Context<UserFundAccountInitialContext>) -> Result<()> {
        UserFundAccountInitialContext::initialize_fund_account(ctx)
    }

    ////////////////////////////////////////////
    // UserFundUpdateContext
    ////////////////////////////////////////////

    pub fn user_update_fund_account_if_needed(ctx: Context<UserFundUpdateContext>) -> Result<()> {
        UserFundUpdateContext::update_fund_account_if_needed(ctx)
    }

    ////////////////////////////////////////////
    // UserFundContext
    ////////////////////////////////////////////

    pub fn user_deposit_sol(
        ctx: Context<UserFundContext>,
        amount: u64,
        metadata: Option<modules::fund::DepositMetadata>,
    ) -> Result<()> {
        UserFundContext::deposit_sol(ctx, amount, metadata)
    }

    pub fn user_request_withdrawal(
        ctx: Context<UserFundContext>,
        receipt_token_amount: u64,
    ) -> Result<()> {
        UserFundContext::request_withdrawal(ctx, receipt_token_amount)
    }

    pub fn user_cancel_withdrawal_request(
        ctx: Context<UserFundContext>,
        request_id: u64,
    ) -> Result<()> {
        UserFundContext::cancel_withdrawal_request(ctx, request_id)
    }

    pub fn user_withdraw(ctx: Context<UserFundContext>, request_id: u64) -> Result<()> {
        UserFundContext::withdraw(ctx, request_id)
    }

    ////////////////////////////////////////////
    // UserFundSupportedTokenContext
    ////////////////////////////////////////////

    pub fn user_deposit_supported_token(
        ctx: Context<UserFundSupportedTokenContext>,
        amount: u64,
        metadata: Option<modules::fund::DepositMetadata>,
    ) -> Result<()> {
        UserFundSupportedTokenContext::deposit_supported_token(ctx, amount, metadata)
    }

    ////////////////////////////////////////////
    // UserRewardInitialContext
    ////////////////////////////////////////////

    pub fn user_initialize_reward_account(ctx: Context<UserRewardInitialContext>) -> Result<()> {
        UserRewardInitialContext::initialize_reward_account(ctx)
    }

    ////////////////////////////////////////////
    // UserRewardContext
    ////////////////////////////////////////////

    pub fn user_update_reward_accounts_if_needed(
        ctx: Context<UserRewardContext>,
        desired_account_size: Option<u32>,
        initialize: bool,
    ) -> Result<()> {
        UserRewardContext::update_accounts_if_needed(ctx, desired_account_size, initialize)
    }

    pub fn user_update_reward_pools(ctx: Context<UserRewardContext>) -> Result<()> {
        UserRewardContext::update_user_reward_pools(ctx)
    }

    pub fn user_claim_rewards(
        ctx: Context<UserRewardContext>,
        reward_pool_id: u8,
        reward_id: u8,
    ) -> Result<()> {
        UserRewardContext::claim_rewards(ctx, reward_pool_id, reward_id)
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

    /// Temporary Instruction to Circulate Assets
    pub fn operator_run(
        ctx: Context<RestakingDepositContext>
    ) -> Result<()> {
        RestakingDepositContext::deposit(ctx, 100, 0)
    }

}
