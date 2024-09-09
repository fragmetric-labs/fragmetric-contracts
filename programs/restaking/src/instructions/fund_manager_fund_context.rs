use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::modules::{common::PDASignerSeeds, fund::FundAccount};

#[derive(Accounts)]
pub struct FundManagerFundContext<'info> {
    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.bump,
        has_one = receipt_token_mint,
    )]
    pub fund_account: Box<Account<'info, FundAccount>>,
}

impl<'info> FundManagerFundContext<'info> {
    pub fn update_sol_capacity_amount(ctx: Context<Self>, capacity_amount: u64) -> Result<()> {
        ctx.accounts
            .fund_account
            .set_sol_capacity_amount(capacity_amount)?;

        Ok(())
    }

    pub fn update_supported_token_capacity_amount(
        ctx: Context<Self>,
        token: Pubkey,
        capacity_amount: u64,
    ) -> Result<()> {
        ctx.accounts
            .fund_account
            .supported_token_mut(token)?
            .set_capacity_amount(capacity_amount)?;

        Ok(())
    }

    pub fn update_withdrawal_enabled_flag(ctx: Context<Self>, enabled: bool) -> Result<()> {
        ctx.accounts
            .fund_account
            .withdrawal_status
            .set_withdrawal_enabled_flag(enabled);

        Ok(())
    }

    pub fn update_sol_withdrawal_fee_rate(
        ctx: Context<Self>,
        sol_withdrawal_fee_rate: u16,
    ) -> Result<()> {
        ctx.accounts
            .fund_account
            .withdrawal_status
            .set_sol_withdrawal_fee_rate(sol_withdrawal_fee_rate);

        Ok(())
    }

    pub fn update_batch_processing_threshold(
        ctx: Context<Self>,
        amount: Option<u64>,
        duration: Option<i64>,
    ) -> Result<()> {
        ctx.accounts
            .fund_account
            .withdrawal_status
            .set_batch_processing_threshold(amount, duration);

        Ok(())
    }
}
