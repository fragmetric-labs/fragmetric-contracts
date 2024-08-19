use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{common::*, constants::*, fund::*};

#[derive(Accounts)]
pub struct FundInitializeFields<'info> {
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

impl<'info> FundInitializeFields<'info> {
    pub fn initialize_sol_withdrawal_fee_rate(
        ctx: Context<Self>,
        sol_withdrawal_fee_rate: u16,
    ) -> Result<()> {
        ctx.accounts
            .fund
            .withdrawal_status
            .set_sol_withdrawal_fee_rate(sol_withdrawal_fee_rate);

        Ok(())
    }

    pub fn initialize_withdrawal_enabled_flag(ctx: Context<Self>, flag: bool) -> Result<()> {
        ctx.accounts
            .fund
            .withdrawal_status
            .set_withdrawal_enabled_flag(flag);

        Ok(())
    }

    pub fn initialize_batch_processing_threshold(
        ctx: Context<Self>,
        amount: u64,
        duration: i64,
    ) -> Result<()> {
        ctx.accounts
            .fund
            .withdrawal_status
            .set_batch_processing_threshold(Some(amount), Some(duration));

        Ok(())
    }
}
