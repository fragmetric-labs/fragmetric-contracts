use anchor_lang::prelude::*;
use fragmetric_util::{request, Upgradable};

use crate::{constants::*, fund::*};

#[derive(Accounts)]
pub struct FundUpdate<'info> {
    #[account(mut, address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        has_one = admin,
        realloc = 8 + Fund::INIT_SPACE,
        realloc::payer = admin,
        realloc::zero = false,
    )]
    pub fund: Account<'info, Fund>,

    pub system_program: Program<'info, System>,
}

impl<'info> FundUpdate<'info> {
    pub fn add_whitelisted_token(
        ctx: Context<Self>,
        request: FundAddWhitelistedTokenRequest,
    ) -> Result<()> {
        let FundAddWhitelistedTokenArgs { token, token_cap } = request.into();
        ctx.accounts
            .fund
            .to_latest_version()
            .add_whitelisted_token(token, token_cap)
    }

    pub fn update_token_info(
        ctx: Context<Self>,
        request: FundUpdateTokenInfoRequest,
    ) -> Result<()> {
        let FundUpdateTokenInfoArgs { token, info } = request.into();
        ctx.accounts
            .fund
            .to_latest_version()
            .update_token(token, info)
    }

    pub fn update_default_protocol_fee_rate(
        ctx: Context<Self>,
        request: FundUpdateDefaultProtocolFeeRateRequest,
    ) -> Result<()> {
        let FundUpdateDefaultProtocolFeeRateArgs {
            default_protocol_fee_rate,
        } = request.into();
        ctx.accounts
            .fund
            .to_latest_version()
            .set_default_protocol_fee_rate(default_protocol_fee_rate)
    }
}

pub struct FundAddWhitelistedTokenArgs {
    pub token: Pubkey,
    pub token_cap: u128,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
#[request(FundAddWhitelistedTokenArgs)]
pub enum FundAddWhitelistedTokenRequest {
    V1(FundAddWhitelistedTokenRequestV1),
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FundAddWhitelistedTokenRequestV1 {
    pub token: Pubkey,
    pub token_cap: u128,
}

impl From<FundAddWhitelistedTokenRequest> for FundAddWhitelistedTokenArgs {
    fn from(value: FundAddWhitelistedTokenRequest) -> Self {
        match value {
            FundAddWhitelistedTokenRequest::V1(value) => Self {
                token: value.token,
                token_cap: value.token_cap,
            },
        }
    }
}

pub struct FundUpdateTokenInfoArgs {
    pub token: Pubkey,
    pub info: TokenInfo,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
#[request(FundUpdateTokenInfoArgs)]
pub enum FundUpdateTokenInfoRequest {
    V1(FundUpdateTokenInfoRequestV1),
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FundUpdateTokenInfoRequestV1 {
    pub token: Pubkey,
    pub info: TokenInfo,
}

impl From<FundUpdateTokenInfoRequest> for FundUpdateTokenInfoArgs {
    fn from(value: FundUpdateTokenInfoRequest) -> Self {
        match value {
            FundUpdateTokenInfoRequest::V1(value) => Self {
                token: value.token,
                info: value.info,
            },
        }
    }
}

pub struct FundUpdateDefaultProtocolFeeRateArgs {
    pub default_protocol_fee_rate: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
#[request(FundUpdateDefaultProtocolFeeRateArgs)]
pub enum FundUpdateDefaultProtocolFeeRateRequest {
    V1(FundUpdateDefaultProtocolFeeRateRequestV1),
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FundUpdateDefaultProtocolFeeRateRequestV1 {
    pub default_protocol_fee_rate: u16,
}

impl From<FundUpdateDefaultProtocolFeeRateRequest> for FundUpdateDefaultProtocolFeeRateArgs {
    fn from(value: FundUpdateDefaultProtocolFeeRateRequest) -> Self {
        match value {
            FundUpdateDefaultProtocolFeeRateRequest::V1(value) => Self {
                default_protocol_fee_rate: value.default_protocol_fee_rate,
            },
        }
    }
}
