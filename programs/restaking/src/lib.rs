use anchor_lang::prelude::*;

pub mod common;
pub mod constants;
pub mod error;
pub mod fund;
pub mod operator;
pub mod reward;
pub mod token;
pub(crate) mod utils;

use common::*;
use fund::*;
use operator::*;
use reward::*;
use token::*;

#[cfg(feature = "mainnet")]
declare_id!("FRAGZZHbvqDwXkqaPSuKocS7EzH7rU7K6h6cW3GQAkEc");
#[cfg(not(feature = "mainnet"))]
declare_id!("fragfP1Z2DXiXNuDYaaCnbGvusMP1DNQswAqTwMuY6e");

#[program]
pub mod restaking {
    use super::*;

    pub fn log_message(ctx: Context<LogMessage>, message: String) -> Result<()> {
        LogMessage::log_message(ctx, message)
    }

    pub fn fund_initialize(ctx: Context<FundInitialize>) -> Result<()> {
        FundInitialize::initialize_fund(ctx)
    }

    pub fn fund_initialize_supported_token(
        ctx: Context<FundInitializeSupportedToken>,
    ) -> Result<()> {
        FundInitializeSupportedToken::initialize_supported_token(ctx)
    }

    pub fn fund_update_sol_capacity_amount(
        ctx: Context<FundUpdate>,
        capacity_amount: u64,
    ) -> Result<()> {
        FundUpdate::update_sol_capacity_amount(ctx, capacity_amount)
    }

    pub fn fund_add_supported_token(
        ctx: Context<FundAddSupportedToken>,
        capacity_amount: u64,
        pricing_source: TokenPricingSource,
    ) -> Result<()> {
        FundAddSupportedToken::add_supported_token(ctx, capacity_amount, pricing_source)
    }

    pub fn fund_update_supported_token(
        ctx: Context<FundUpdate>,
        token: Pubkey,
        capacity_amount: u64,
    ) -> Result<()> {
        FundUpdate::update_supported_token(ctx, token, capacity_amount)
    }

    pub fn fund_update_sol_withdrawal_fee_rate(
        ctx: Context<FundUpdate>,
        sol_withdrawal_fee_rate: u16,
    ) -> Result<()> {
        FundUpdate::update_sol_withdrawal_fee_rate(ctx, sol_withdrawal_fee_rate)
    }

    pub fn fund_update_withdrawal_enabled_flag(ctx: Context<FundUpdate>, flag: bool) -> Result<()> {
        FundUpdate::update_withdrawal_enabled_flag(ctx, flag)
    }

    pub fn fund_update_batch_processing_threshold(
        ctx: Context<FundUpdate>,
        amount: Option<u64>,
        duration: Option<i64>,
    ) -> Result<()> {
        FundUpdate::update_batch_processing_threshold(ctx, amount, duration)
    }

    pub fn fund_update_price(ctx: Context<FundUpdatePrice>) -> Result<()> {
        FundUpdatePrice::update_price(ctx)
    }

    pub fn fund_initialize_user_accounts(ctx: Context<FundInitializeUserAccounts>) -> Result<()> {
        FundInitializeUserAccounts::initialize_user_accounts(ctx)
    }

    pub fn fund_deposit_sol(
        ctx: Context<FundDepositSOL>,
        amount: u64,
        metadata: Option<Metadata>,
    ) -> Result<()> {
        FundDepositSOL::deposit_sol(ctx, amount, metadata)
    }

    pub fn fund_deposit_token(
        ctx: Context<FundDepositToken>,
        amount: u64,
        metadata: Option<Metadata>,
    ) -> Result<()> {
        FundDepositToken::deposit_token(ctx, amount, metadata)
    }

    pub fn fund_request_withdrawal(
        ctx: Context<FundRequestWithdrawal>,
        receipt_token_amount: u64,
    ) -> Result<()> {
        FundRequestWithdrawal::request_withdrawal(ctx, receipt_token_amount)
    }

    pub fn fund_cancel_withdrawal_request(
        ctx: Context<FundCancelWithdrawalRequest>,
        request_id: u64,
    ) -> Result<()> {
        FundCancelWithdrawalRequest::cancel_withdrawal_request(ctx, request_id)
    }

    pub fn fund_withdraw(ctx: Context<FundWithdraw>, request_id: u64) -> Result<()> {
        FundWithdraw::withdraw(ctx, request_id)
    }

    pub fn operator_run_if_needed(ctx: Context<OperatorRunIfNeeded>) -> Result<()> {
        OperatorRunIfNeeded::operator_run_if_needed(ctx)
    }

    pub fn operator_run(ctx: Context<OperatorRun>) -> Result<()> {
        OperatorRun::operator_run(ctx)
    }

    pub fn reward_add_holder(
        ctx: Context<RewardAddHolder>,
        name: String,
        description: String,
        pubkeys: Vec<Pubkey>,
    ) -> Result<()> {
        RewardAddHolder::add_holder(ctx, name, description, pubkeys)
    }

    pub fn reward_add_reward(
        ctx: Context<RewardAddReward>,
        name: String,
        description: String,
        reward_type: RewardType,
    ) -> Result<()> {
        RewardAddReward::add_reward(ctx, name, description, reward_type)
    }

    pub fn reward_add_reward_pool(
        ctx: Context<RewardAddRewardPool>,
        name: String,
        holder_id: Option<u8>,
        custom_contribution_accrual_rate_enabled: bool,
    ) -> Result<()> {
        RewardAddRewardPool::add_reward_pool(
            ctx,
            name,
            holder_id,
            custom_contribution_accrual_rate_enabled,
        )
    }

    pub fn reward_claim_user_rewards(
        ctx: Context<RewardClaimUserRewards>,
        reward_pool_id: u8,
        reward_id: u8,
    ) -> Result<()> {
        RewardClaimUserRewards::claim_user_rewards(ctx, reward_pool_id, reward_id)
    }

    pub fn reward_close_reward_pool(
        ctx: Context<RewardCloseRewardPool>,
        reward_pool_id: u8,
    ) -> Result<()> {
        RewardCloseRewardPool::close_reward_pool(ctx, reward_pool_id)
    }

    pub fn reward_initialize(ctx: Context<RewardInitialize>) -> Result<()> {
        RewardInitialize::initialize_reward(ctx)
    }

    pub fn reward_settle(
        ctx: Context<RewardSettle>,
        reward_pool_id: u8,
        reward_id: u8,
        amount: u64,
    ) -> Result<()> {
        RewardSettle::settle_reward(ctx, reward_pool_id, reward_id, amount)
    }

    pub fn reward_update_reward_pools(ctx: Context<RewardUpdateRewardPools>) -> Result<()> {
        RewardUpdateRewardPools::update_reward_pools(ctx)
    }

    pub fn reward_update_user_reward_pools(
        ctx: Context<RewardUpdateUserRewardPools>,
    ) -> Result<()> {
        RewardUpdateUserRewardPools::update_user_reward_pools(ctx)
    }

    pub fn token_initialize_payer_account(ctx: Context<TokenInitializePayerAccount>) -> Result<()> {
        TokenInitializePayerAccount::initialize_payer_account(ctx)
    }

    pub fn token_add_payer_account_lamports(
        ctx: Context<TokenInitializePayerAccount>,
        amount: u64,
    ) -> Result<()> {
        TokenInitializePayerAccount::add_payer_account_lamports(ctx, amount)
    }

    pub fn token_set_receipt_token_mint_authority(
        ctx: Context<TokenSetReceiptTokenMintAuthority>,
    ) -> Result<()> {
        TokenSetReceiptTokenMintAuthority::set_receipt_token_mint_authority(ctx)
    }

    #[interface(spl_transfer_hook_interface::initialize_extra_account_meta_list)]
    pub fn token_initialize_extra_account_meta_list(
        ctx: Context<TokenInitializeExtraAccountMetaList>,
    ) -> Result<()> {
        TokenInitializeExtraAccountMetaList::initialize_extra_account_meta_list(ctx)
    }

    pub fn token_update_extra_account_meta_list(
        ctx: Context<TokenInitializeExtraAccountMetaList>,
    ) -> Result<()> {
        TokenInitializeExtraAccountMetaList::update_extra_account_meta_list(ctx)
    }

    #[interface(spl_transfer_hook_interface::execute)]
    pub fn token_transfer_hook(ctx: Context<TokenTransferHook>, amount: u64) -> Result<()> {
        TokenTransferHook::transfer_hook(ctx, amount)
    }
}
