use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{constants::*, error::ErrorCode, fund::*};

#[derive(Accounts)]
pub struct FundUpdate<'info> {
    #[account(mut, address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [FUND_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub fund: Box<Account<'info, Fund>>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,
}

impl<'info> FundUpdate<'info> {
    pub fn add_whitelisted_token(ctx: Context<Self>, token: Pubkey, token_cap: u64) -> Result<()> {
        ctx.accounts.fund.check_token_does_not_exist(&token)?;
        ctx.accounts.fund.add_whitelisted_token(token, token_cap);

        Ok(())
    }

    pub fn update_whitelisted_token(
        ctx: Context<Self>,
        token: Pubkey,
        token_cap: u64,
    ) -> Result<()> {
        ctx.accounts
            .fund
            .whitelisted_token_mut(token)
            .ok_or_else(|| error!(ErrorCode::FundNotExistingToken))?
            .update(token_cap);

        Ok(())
    }

    pub fn update_sol_withdrawal_fee_rate(
        ctx: Context<Self>,
        sol_withdrawal_fee_rate: u16,
    ) -> Result<()> {
        ctx.accounts
            .fund
            .withdrawal_status
            .set_sol_withdrawal_fee_rate(sol_withdrawal_fee_rate);

        Ok(())
    }

    pub fn update_withdrawal_enabled_flag(ctx: Context<Self>, flag: bool) -> Result<()> {
        ctx.accounts
            .fund
            .withdrawal_status
            .set_withdrawal_enabled_flag(flag);

        Ok(())
    }

    pub fn update_batch_processing_threshold(
        ctx: Context<Self>,
        amount: Option<u64>,
        duration: Option<i64>,
    ) -> Result<()> {
        ctx.accounts
            .fund
            .withdrawal_status
            .set_batch_processing_threshold(amount, duration);

        Ok(())
    }
}
