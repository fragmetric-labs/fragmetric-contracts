use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use fragmetric_util::{request, Upgradable};

use crate::{constants::*, fund::*};

#[derive(Accounts)]
pub struct FundInitializeOthers<'info> {
    #[account(mut, address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [FUND_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub fund: Account<'info, Fund>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,
}

impl<'info> FundInitializeOthers<'info> {
    pub fn initialize_sol_withdrawal_fee_rate(ctx: Context<Self>, request: FundInitializeSolWithdrawalFeeRateRequest) -> Result<()> {
        let args = FundInitializeSolWithdrawalFeeRateArgs::from(request);

        ctx.accounts.fund
            .to_latest_version()
            .set_sol_withdrawal_fee_rate(args.sol_withdrawal_fee_rate)
    }

    pub fn initialize_whitelisted_tokens(ctx: Context<Self>, request: FundInitializeWhitelistedTokensRequest) -> Result<()> {
        let args = FundInitializeWhitelistedTokensArgs::from(request);

        ctx.accounts.fund
            .to_latest_version()
            .set_whitelisted_tokens(args.whitelisted_tokens)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
#[request(FundInitializeSolWithdrawalFeeRateArgs)]
pub enum FundInitializeSolWithdrawalFeeRateRequest {
    V1(FundInitializeSolWithdrawalFeeRateRequestV1),
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FundInitializeSolWithdrawalFeeRateRequestV1 {
    pub sol_withdrawal_fee_rate: u16,
}

pub struct FundInitializeSolWithdrawalFeeRateArgs {
    pub sol_withdrawal_fee_rate: u16,
}

impl From<FundInitializeSolWithdrawalFeeRateRequest> for FundInitializeSolWithdrawalFeeRateArgs {
    fn from(value: FundInitializeSolWithdrawalFeeRateRequest) -> Self {
        match value {
            FundInitializeSolWithdrawalFeeRateRequest::V1(value) => Self {
                sol_withdrawal_fee_rate: value.sol_withdrawal_fee_rate,
            },
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
#[request(FundInitializeWhitelistedTokensArgs)]
pub enum FundInitializeWhitelistedTokensRequest {
    V1(FundInitializeWhitelistedTokensRequestV1),
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FundInitializeWhitelistedTokensRequestV1 {
    pub whitelisted_tokens: Vec<TokenInfo>,
}

pub struct FundInitializeWhitelistedTokensArgs {
    pub whitelisted_tokens: Vec<TokenInfo>,
}

impl From<FundInitializeWhitelistedTokensRequest> for FundInitializeWhitelistedTokensArgs {
    fn from(value: FundInitializeWhitelistedTokensRequest) -> Self {
        match value {
            FundInitializeWhitelistedTokensRequest::V1(value) => Self {
                whitelisted_tokens: value.whitelisted_tokens,
            },
        }
    }
}
