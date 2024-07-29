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
// declare_id!("fragfP1Z2DXiXNuDYaaCnbGvusMP1DNQswAqTwMuY6e");
declare_id!("9UpfJBgVKuZ1EzowJL6qgkYVwv3HhLpo93aP8L1QW86D");

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

    pub fn token_mint_receipt_token_for_test(
        ctx: Context<MintReceiptToken>,
        amount: u64,
    ) -> Result<()> {
        MintReceiptToken::mint_receipt_token_for_test(ctx, amount)
    }

    #[interface(spl_transfer_hook_interface::initialize_extra_account_meta_list)]
    pub fn token_initialize_extra_account_meta_list(
        ctx: Context<InitializeExtraAccountMetaList>,
    ) -> Result<()> {
        InitializeExtraAccountMetaList::initialize_extra_account_meta_list(ctx)
    }

    #[interface(spl_transfer_hook_interface::execute)]
    pub fn token_transfer_hook(
        ctx: Context<FragSOLTransferHook>,
        amount: u64,
    ) -> Result<()> {
        FragSOLTransferHook::transfer_hook(ctx, amount)
    }
}
