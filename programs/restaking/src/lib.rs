use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod fund;
// pub mod oracle;

use fund::*;
// use oracle::*;

#[cfg(feature = "mainnet")]
declare_id!("FRAGZZHbvqDwXkqaPSuKocS7EzH7rU7K6h6cW3GQAkEc");
#[cfg(not(feature = "mainnet"))]
// declare_id!("fragfP1Z2DXiXNuDYaaCnbGvusMP1DNQswAqTwMuY6e");
declare_id!("9UpfJBgVKuZ1EzowJL6qgkYVwv3HhLpo93aP8L1QW86D");

#[program]
pub mod restaking {
    use super::*;

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
}
