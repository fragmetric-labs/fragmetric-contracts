use anchor_lang::prelude::*;

use crate::fund::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct FundUpdateToken<'info> {
    #[account(mut, address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(mut)]
    pub fund: Account<'info, Fund>,
}

impl<'info> FundUpdateToken<'info> {
    pub fn add_whitelisted_token(
        ctx: Context<Self>,
        token: Pubkey,
        token_cap: u64
    ) -> Result<()> {
        let fund = &mut ctx.accounts.fund;

        Ok(fund.add_whitelisted_token(token, token_cap)?)
    }

    pub fn update_token_info(
        ctx: Context<Self>,
        token: Pubkey,
        info: TokenInfo
    ) -> Result<()> {
        let fund = &mut ctx.accounts.fund;

        Ok(fund.update_token(token, info)?)
    }

    pub fn update_default_protocol_fee_rate(
        ctx: Context<Self>,
        default_protocol_fee_rate: u16,
    ) -> Result<()> {
        let fund = &mut ctx.accounts.fund;

        Ok(fund.update_default_protocol_fee_rate(default_protocol_fee_rate)?)
    }
}
