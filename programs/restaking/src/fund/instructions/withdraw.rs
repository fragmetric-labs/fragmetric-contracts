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

    // TODO: use address lookup table!
    #[account(address = BSOL_STAKE_POOL_ADDRESS)]
    /// CHECK: will be checked and deserialized when needed
    pub token_pricing_source_0: UncheckedAccount<'info>,

    // TODO: use address lookup table!
    #[account(address = MSOL_STAKE_POOL_ADDRESS)]
    /// CHECK: will be checked and deserialized when needed
    pub token_pricing_source_1: UncheckedAccount<'info>,
}

impl<'info> FundWithdraw<'info> {
    pub fn withdraw(mut ctx: Context<Self>, request_id: u64) -> Result<()> {
        let fund = &mut ctx.accounts.fund;

        // Verify
        require_gt!(fund.withdrawal_status.next_request_id, request_id);

        // Step 1: Update price
        let sources = [
            ctx.accounts.token_pricing_source_0.as_ref(),
            ctx.accounts.token_pricing_source_1.as_ref(),
        ];
        fund.update_token_prices(&sources)?;

        // Step 2: Complete withdrawal request
        fund.withdrawal_status.check_withdrawal_enabled()?;
        let request = ctx
            .accounts
            .user_receipt
            .pop_withdrawal_request(request_id)?;
        fund.withdrawal_status
            .check_batch_processing_completed(request.batch_id)?;

        // Step 3: Calculate withdraw amount
        let receipt_token_total_supply = ctx.accounts.receipt_token_mint.supply;
        let receipt_token_price = fund.receipt_token_price(
            ctx.accounts.receipt_token_mint.decimals,
            receipt_token_total_supply,
        )?;
        let sol_amount = fund.calculate_sol_from_receipt_tokens(
            request.receipt_token_amount,
            receipt_token_total_supply,
        )?;
        let sol_fee_amount = fund
            .withdrawal_status
            .calculate_sol_withdrawal_fee(sol_amount)?;
        let sol_withdraw_amount = sol_amount
            .checked_sub(sol_fee_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;

        // Step 4: Withdraw
        fund.withdrawal_status.withdraw(sol_withdraw_amount)?;
        Self::transfer_sol(&mut ctx, sol_withdraw_amount)
            .map_err(|_| error!(ErrorCode::FundSOLTransferFailed))?;
        // TODO transfer sol fee to treasury fund

        emit!(UserWithdrawnSOLFromFund {
            user: ctx.accounts.user.key(),
            user_receipt: Clone::clone(&ctx.accounts.user_receipt),
            request_id,
            receipt_token_mint: ctx.accounts.receipt_token_mint.key(),
            receipt_token_amount: request.receipt_token_amount,
            sol_withdraw_amount,
            sol_fee_amount,
            fund_info: FundInfo::new_from_fund(
                ctx.accounts.fund.as_ref(),
                receipt_token_price,
                receipt_token_total_supply
            ),
        });

        Ok(())
    }

    fn transfer_sol(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts.fund.sub_lamports(amount)?;
        ctx.accounts.user.add_lamports(amount)?;

        Ok(())
    }
}
