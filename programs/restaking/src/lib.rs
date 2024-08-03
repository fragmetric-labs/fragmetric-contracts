use anchor_lang::prelude::*;

pub mod common;
pub mod constants;
pub mod error;
pub mod fund;
pub mod token;
// pub mod oracle;

use common::*;
use fund::*;
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

    pub fn fund_initialize(
        ctx: Context<FundInitialize>,
        request: FundInitializeRequest,
    ) -> Result<()> {
        FundInitialize::initialize_fund(ctx, request)
    }

    pub fn fund_add_whitelisted_token(
        ctx: Context<FundUpdate>,
        request: FundAddWhitelistedTokenRequest,
    ) -> Result<()> {
        FundUpdate::add_whitelisted_token(ctx, request)
    }

    pub fn fund_update_token_info(
        ctx: Context<FundUpdate>,
        request: FundUpdateTokenInfoRequest,
    ) -> Result<()> {
        FundUpdate::update_token_info(ctx, request)
    }

    pub fn fund_update_default_protocol_fee_rate(
        ctx: Context<FundUpdate>,
        request: FundUpdateDefaultProtocolFeeRateRequest,
    ) -> Result<()> {
        FundUpdate::update_default_protocol_fee_rate(ctx, request)
    }

    pub fn fund_update_withdrawal_enabled_flag(ctx: Context<FundUpdate>, flag: bool) -> Result<()> {
        FundUpdate::update_withdrawal_enabled_flag(ctx, flag)
    }

    pub fn fund_update_batch_processing_threshold(
        ctx: Context<FundUpdate>,
        amount: u128,
        duration: i64,
    ) -> Result<()> {
        FundUpdate::update_batch_processing_threshold(ctx, amount, duration)
    }

    pub fn fund_deposit_sol(
        ctx: Context<FundDepositSOL>,
        request: FundDepositSOLRequest,
    ) -> Result<()> {
        FundDepositSOL::deposit_sol(ctx, request)
    }

    pub fn fund_deposit_token(
        ctx: Context<FundDepositToken>,
        request: FundDepositTokenRequest,
    ) -> Result<()> {
        FundDepositToken::deposit_token(ctx, request)
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

    // for test
    pub fn fund_process_withdrawal_requests_for_test(
        ctx: Context<FundProcessWithdrawalRequestsForTest>,
    ) -> Result<()> {
        FundProcessWithdrawalRequestsForTest::process_withdrawal_requests_for_test(ctx)
    }

    pub fn fund_withdraw_sol(ctx: Context<FundWithdrawSOL>, request_id: u64) -> Result<()> {
        FundWithdrawSOL::withdraw_sol(ctx, request_id)
    }

    // for test
    pub fn token_mint_receipt_token_for_test(
        ctx: Context<TokenMintReceiptToken>,
        amount: u64,
    ) -> Result<()> {
        TokenMintReceiptToken::mint_receipt_token_for_test(ctx, amount)
    }

    #[interface(spl_transfer_hook_interface::initialize_extra_account_meta_list)]
    pub fn token_initialize_extra_account_meta_list(
        ctx: Context<TokenInitializeExtraAccountMetaList>,
    ) -> Result<()> {
        TokenInitializeExtraAccountMetaList::initialize_extra_account_meta_list(ctx)
    }

    #[interface(spl_transfer_hook_interface::execute)]
    pub fn token_transfer_hook(ctx: Context<TokenTransferHook>, amount: u64) -> Result<()> {
        TokenTransferHook::transfer_hook(ctx, amount)
    }
}
