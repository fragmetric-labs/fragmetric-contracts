use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{common::*, constants::*, error::ErrorCode, fund::*};

#[derive(Accounts)]
pub struct FundUpdate<'info> {
    #[account(mut, address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [Fund::SEED, receipt_token_mint.key().as_ref()],
        bump = fund.bump,
        has_one = receipt_token_mint,
    )]
    pub fund: Box<Account<'info, Fund>>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,
}

impl<'info> FundUpdate<'info> {
    pub fn update_supported_token(
        ctx: Context<Self>,
        token: Pubkey,
        capacity_amount: u64,
    ) -> Result<()> {
        ctx.accounts
            .fund
            .supported_token_mut(token)
            .ok_or_else(|| error!(ErrorCode::FundNotExistingToken))?
            .update(capacity_amount);

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
