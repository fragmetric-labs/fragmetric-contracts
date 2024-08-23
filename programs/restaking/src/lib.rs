use anchor_lang::prelude::*;

pub mod common;
pub mod constants;
pub mod error;
pub mod fund;
pub mod operator;
pub mod token;
pub(crate) mod utils;
// pub mod oracle;

use common::*;
use fund::*;
use operator::*;
use token::*;
// use oracle::*;

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

    pub fn fund_initialize_token(ctx: Context<FundInitializeToken>) -> Result<()> {
        FundInitializeToken::initialize(ctx)
    }

    pub fn fund_add_supported_token(
        ctx: Context<FundAddSupportedToken>,
        token_cap: u64,
        token_pricing_source: TokenPricingSource,
    ) -> Result<()> {
        FundAddSupportedToken::add_supported_token(ctx, token_cap, token_pricing_source)
    }

    pub fn fund_update_supported_token(
        ctx: Context<FundUpdate>,
        token: Pubkey,
        token_cap: u64,
    ) -> Result<()> {
        FundUpdate::update_supported_token(ctx, token, token_cap)
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
