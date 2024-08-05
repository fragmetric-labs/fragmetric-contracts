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

    pub fn update_sol_withdrawal_fee_rate(
        ctx: Context<Self>,
        request: FundUpdateSolWithdrawalFeeRateRequest,
    ) -> Result<()> {
        let FundUpdateSolWithdrawalFeeRateArgs {
            sol_withdrawal_fee_rate,
        } = request.into();
        ctx.accounts
            .fund
            .to_latest_version()
            .withdrawal_status
            .set_sol_withdrawal_fee_rate(sol_withdrawal_fee_rate)
    }

    pub fn update_withdrawal_enabled_flag(ctx: Context<Self>, flag: bool) -> Result<()> {
        ctx.accounts
            .fund
            .to_latest_version()
            .withdrawal_status
            .set_withdrawal_enabled_flag(flag)
    }

    pub fn update_batch_processing_threshold(
        ctx: Context<Self>,
        amount: Option<u128>,
        duration: Option<i64>,
    ) -> Result<()> {
        ctx.accounts
            .fund
            .to_latest_version()
            .withdrawal_status
            .set_batch_processing_threshold(amount, duration)
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

pub struct FundUpdateSolWithdrawalFeeRateArgs {
    pub sol_withdrawal_fee_rate: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
#[request(FundUpdateSolWithdrawalFeeRateArgs)]
pub enum FundUpdateSolWithdrawalFeeRateRequest {
    V1(FundUpdateSolWithdrawalFeeRateRequestV1),
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FundUpdateSolWithdrawalFeeRateRequestV1 {
    pub sol_withdrawal_fee_rate: u16,
}

impl From<FundUpdateSolWithdrawalFeeRateRequest> for FundUpdateSolWithdrawalFeeRateArgs {
    fn from(value: FundUpdateSolWithdrawalFeeRateRequest) -> Self {
        match value {
            FundUpdateSolWithdrawalFeeRateRequest::V1(value) => Self {
                sol_withdrawal_fee_rate: value.sol_withdrawal_fee_rate,
            },
        }
    }
}
