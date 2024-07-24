use anchor_lang::prelude::*;

use crate::{constants::*, fund::*};

#[derive(Accounts)]
pub struct FundUpdate<'info> {
    #[account(mut, address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(mut)]
    pub fund: Account<'info, Fund>,
}

impl<'info> FundUpdate<'info> {
    pub fn add_whitelisted_token(
        ctx: Context<Self>,
        request: FundAddWhitelistedTokenRequest,
    ) -> Result<()> {
        let FundAddWhitelistedTokenRequest { token, token_cap } = request;
        ctx.accounts.fund.add_whitelisted_token(token, token_cap)
    }

    pub fn update_token_info(
        ctx: Context<Self>,
        request: FundUpdateTokenInfoRequest,
    ) -> Result<()> {
        ctx.accounts.fund.update_token(request.token, request.info)
    }

    pub fn update_default_protocol_fee_rate(
        ctx: Context<Self>,
        request: FundUpdateDefaultProtocolFeeRateRequest,
    ) -> Result<()> {
        ctx.accounts
            .fund
            .set_default_protocol_fee_rate(request.default_protocol_fee_rate)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FundAddWhitelistedTokenRequest {
    pub token: Pubkey,
    pub token_cap: u128,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FundUpdateTokenInfoRequest {
    pub token: Pubkey,
    pub info: TokenInfo,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FundUpdateDefaultProtocolFeeRateRequest {
    pub default_protocol_fee_rate: u16,
}
