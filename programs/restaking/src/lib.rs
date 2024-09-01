use anchor_lang::prelude::*;

pub(crate) mod constants;
pub(crate) mod errors;
pub(crate) mod events;
pub(crate) mod utils;
pub(crate) mod modules;
mod instructions;

use instructions::*;

#[cfg(feature = "mainnet")]
declare_id!("fragnAis7Bp6FTsMoa6YcH8UffhEw43Ph79qAiK3iF3");
#[cfg(not(feature = "mainnet"))]
declare_id!("frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ");

#[program]
pub mod restaking {
    use super::*;

    /** AdminEmptyContext **/
    pub fn log_message(ctx: Context<AdminEmptyContext>, message: String) -> Result<()> {
        AdminEmptyContext::log_message(ctx, message)
    }


    /** AdminFundContext **/
    pub fn admin_initialize_fund_accounts_if_needed(ctx: Context<AdminFundContext>) -> Result<()> {
        AdminFundContext::initialize_fund_accounts_if_needed(ctx)
    }

    pub fn admin_transfer_receipt_token_mint_authority(ctx: Context<AdminFundContext>) -> Result<()> {
        AdminFundContext::transfer_receipt_token_mint_authority(ctx)
    }


    /** AdminReceiptTokenMintContext **/
    #[interface(spl_transfer_hook_interface::initialize_extra_account_meta_list)]
    pub fn admin_initialize_receipt_token_mint_extra_account_meta_list(
        ctx: Context<AdminReceiptTokenMintContext>,
    ) -> Result<()> {
        AdminReceiptTokenMintContext::initialize_extra_account_meta_list(ctx)
    }

    pub fn admin_update_receipt_token_mint_extra_account_meta_list(
        ctx: Context<AdminReceiptTokenMintContext>,
    ) -> Result<()> {
        AdminReceiptTokenMintContext::update_extra_account_meta_list(ctx)
    }


    /** AdminRewardContext **/
    pub fn admin_initialize_reward_account_if_needed(ctx: Context<AdminRewardContext>) -> Result<()> {
        AdminRewardContext::initialize_reward_account_if_needed(ctx)
    }

    pub fn admin_realloc_reward_account_if_needed(ctx: Context<AdminRewardContext>, required_size: Option<u32>, assert: bool) -> Result<()> {
        AdminRewardContext::realloc_reward_account_if_needed(ctx, required_size, assert)
    }

    pub fn admin_update_reward_pools(ctx: Context<AdminRewardContext>) -> Result<()> {
        AdminRewardContext::update_reward_pools(ctx)
    }


    /** FundManagerFundContext **/
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


    /** FundManagerFundSupportedTokenContext **/
    pub fn fund_manager_add_supported_token(
        ctx: Context<FundManagerFundSupportedTokenContext>,
        capacity_amount: u64,
        pricing_source: modules::fund::TokenPricingSource,
    ) -> Result<()> {
        FundManagerFundSupportedTokenContext::add_supported_token(ctx, capacity_amount, pricing_source)
    }


    /** FundManagerRewardContext **/
    pub fn fund_manager_add_reward(
        ctx: Context<FundManagerRewardContext>,
        name: String,
        description: String,
        reward_type: modules::reward::RewardType,
    ) -> Result<()> {
        FundManagerRewardContext::add_reward(ctx, name, description, reward_type)
    }

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
        FundManagerRewardContext::add_reward_pool(ctx, name, holder_id, custom_contribution_accrual_rate_enabled)
    }

    pub fn fund_manager_close_reward_pool(
        ctx: Context<FundManagerRewardContext>,
        reward_pool_id: u8,
    ) -> Result<()> {
        FundManagerRewardContext::close_reward_pool(ctx, reward_pool_id)
    }

    pub fn fund_manager_settle_reward(
        ctx: Context<FundManagerRewardContext>,
        reward_pool_id: u8,
        reward_id: u8,
        amount: u64,
    ) -> Result<()> {
        FundManagerRewardContext::settle_reward(ctx, reward_pool_id, reward_id, amount)
    }


    /** OperatorFundContext **/
    pub fn operator_process_fund_withdrawal_job(ctx: Context<OperatorFundContext>, forced: bool) -> Result<()> {
        OperatorFundContext::process_fund_withdrawal_job(ctx, forced)
    }


    /** UserFundContext **/
    pub fn user_initialize_user_accounts_if_needed(ctx: Context<UserFundContext>) -> Result<()> {
        UserFundContext::initialize_user_accounts_if_needed(ctx)
    }

    pub fn user_update_prices(ctx: Context<UserFundContext>) -> Result<()> {
        UserFundContext::update_prices(ctx)
    }

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


    /** UserFundSupportedTokenContext **/
    pub fn user_deposit_supported_token(
        ctx: Context<UserFundSupportedTokenContext>,
        amount: u64,
        metadata: Option<modules::fund::DepositMetadata>,
    ) -> Result<()> {
        UserFundSupportedTokenContext::deposit_supported_token(ctx, amount, metadata)
    }


    /** UserRewardContext **/
    pub fn user_update_reward_pools(
        ctx: Context<UserRewardContext>,
    ) -> Result<()> {
        UserRewardContext::update_user_reward_pools(ctx)
    }

    pub fn user_claim_rewards(
        ctx: Context<UserRewardContext>,
        reward_pool_id: u8,
        reward_id: u8,
    ) -> Result<()> {
        UserRewardContext::claim_rewards(ctx, reward_pool_id, reward_id)
    }


    /** UserReceiptTokenTransferContext **/
    #[interface(spl_transfer_hook_interface::execute)]
    pub fn token_transfer_hook(ctx: Context<UserReceiptTokenTransferContext>, amount: u64) -> Result<()> {
        UserReceiptTokenTransferContext::handle_transfer(ctx, amount)
    }
}
