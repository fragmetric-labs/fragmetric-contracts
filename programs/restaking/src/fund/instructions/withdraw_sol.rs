use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use fragmetric_util::Upgradable;

use crate::{constants::*, error::ErrorCode, fund::*};

#[derive(Accounts)]
pub struct FundWithdrawSOL<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [USER_RECEIPT_SEED, receipt_token_mint.key().as_ref()],
        bump,
        realloc = 8 + UserReceipt::INIT_SPACE,
        realloc::payer = user,
        realloc::zero = false,
    )]
    pub user_receipt: Account<'info, UserReceipt>,

    #[account(
        mut,
        seeds = [FUND_SEED, receipt_token_mint.key().as_ref()],
        bump,
        realloc = 8 + Fund::INIT_SPACE,
        // TODO must paid by fund
        realloc::payer = user,
        realloc::zero = false,
    )]
    pub fund: Account<'info, Fund>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub system_program: Program<'info, System>,
}

impl<'info> FundWithdrawSOL<'info> {
    pub fn withdraw_sol(mut ctx: Context<Self>, request_id: u64) -> Result<()> {
        let request = ctx
            .accounts
            .user_receipt
            .pop_withdrawal_request(request_id)?;

        let sol_amount = Self::get_sol_amount_by_exchange_rate(&ctx, request.receipt_token_amount)?;
        let sol_withdraw_amount = ctx
            .accounts
            .fund
            .to_latest_version()
            .withdrawal_status
            .withdraw_sol(request.batch_id, sol_amount)?;
        let sol_fee_amount = sol_amount - sol_withdraw_amount;

        Self::transfer_sol(&mut ctx, sol_withdraw_amount)
            .map_err(|_| error!(ErrorCode::FundSOLTransferFailed))?;

        let admin = ctx.accounts.fund.admin;
        let receipt_token_mint = ctx.accounts.fund.receipt_token_mint;
        let fund = ctx.accounts.fund.to_latest_version();
        emit!(FundSOLWithdrawn {
            user: ctx.accounts.user.key(),
            user_receipt: Clone::clone(&ctx.accounts.user_receipt),
            request_id,
            lrt_mint: ctx.accounts.receipt_token_mint.key(),
            lrt_amount: request.receipt_token_amount,
            sol_withdraw_amount,
            sol_fee_amount,
            fund_info: fund.to_info(admin, receipt_token_mint),
        });

        Ok(())
    }

    #[allow(unused_variables)]
    fn get_sol_amount_by_exchange_rate(ctx: &Context<Self>, amount: u64) -> Result<u64> {
        Ok(amount)
    }

    fn transfer_sol(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts.fund.sub_lamports(amount)?;
        ctx.accounts.user.add_lamports(amount)?;

        Ok(())
    }
}
