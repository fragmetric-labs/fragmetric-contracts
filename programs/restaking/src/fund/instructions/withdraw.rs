use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{common::*, constants::*, error::ErrorCode, fund::*};

#[derive(Accounts)]
pub struct FundWithdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [UserReceipt::SEED, user.key().as_ref(), receipt_token_mint.key().as_ref()],
        bump = user_receipt.bump,
        has_one = user,
        has_one = receipt_token_mint,
    )]
    pub user_receipt: Account<'info, UserReceipt>,

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

impl<'info> FundWithdraw<'info> {
    pub fn withdraw(mut ctx: Context<Self>, request_id: u64) -> Result<()> {
        let request = ctx
            .accounts
            .user_receipt
            .pop_withdrawal_request(request_id)?;

        let sol_amount = Self::get_sol_amount_by_exchange_rate(&ctx, request.receipt_token_amount)?;

        let withdrawal_status = &mut ctx.accounts.fund.withdrawal_status;
        withdrawal_status.check_withdrawal_enabled()?;
        withdrawal_status.check_batch_processing_completed(request.batch_id)?;

        let sol_fee_amount = withdrawal_status.calculate_sol_withdrawal_fee(sol_amount)?;
        let sol_withdraw_amount = sol_amount
            .checked_sub(sol_fee_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
        withdrawal_status.withdraw(sol_withdraw_amount)?;

        Self::transfer_sol(&mut ctx, sol_withdraw_amount)
            .map_err(|_| error!(ErrorCode::FundSOLTransferFailed))?;

        emit!(FundSOLWithdrawn {
            user: ctx.accounts.user.key(),
            user_receipt: Clone::clone(&ctx.accounts.user_receipt),
            request_id,
            lrt_mint: ctx.accounts.receipt_token_mint.key(),
            lrt_amount: request.receipt_token_amount,
            sol_withdraw_amount,
            sol_fee_amount,
            fund_info: FundInfo::new_from_fund(ctx.accounts.fund.as_ref()),
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
