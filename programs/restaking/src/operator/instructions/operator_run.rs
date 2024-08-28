use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

use crate::{common::*, constants::*, error::ErrorCode, fund::*, operator::*, token::*};

#[derive(Accounts)]
pub struct OperatorRun<'info> {
    // Only the admin can run the operator manually.
    #[account(mut, address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [Fund::SEED, receipt_token_mint.key().as_ref()],
        bump = fund.bump,
        has_one = receipt_token_mint,
    )]
    pub fund: Box<Account<'info, Fund>>,

    #[account(
        seeds = [ReceiptTokenLockAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump = receipt_token_lock_authority.bump,
        has_one = receipt_token_mint,
    )]
    pub receipt_token_lock_authority: Account<'info, ReceiptTokenLockAuthority>,

    #[account(mut, address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        token::mint = receipt_token_mint,
        token::authority = receipt_token_lock_authority,
        seeds = [RECEIPT_TOKEN_LOCK_ACCOUNT_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>, // fund's fragSOL lock account

    // TODO: use address lookup table!
    #[account(address = BSOL_STAKE_POOL_ADDRESS)]
    /// CHECK: will be checked and deserialized when needed
    pub token_pricing_source_0: UncheckedAccount<'info>,

    // TODO: use address lookup table!
    #[account(address = MSOL_STAKE_POOL_ADDRESS)]
    /// CHECK: will be checked and deserialized when needed
    pub token_pricing_source_1: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
}

impl<'info> OperatorRun<'info> {
    /// Manually run the operator.
    /// This instruction is only available to ADMIN
    pub fn operator_run(mut ctx: Context<Self>) -> Result<()> {
        let withdrawal_status = &mut ctx.accounts.fund.withdrawal_status;

        withdrawal_status.start_processing_pending_batch_withdrawal()?;

        let fund = &mut ctx.accounts.fund;
        let sources = [
            ctx.accounts.token_pricing_source_0.as_ref(),
            ctx.accounts.token_pricing_source_1.as_ref(),
        ];
        fund.update_token_prices(&sources)?;
        let total_sol_value_in_fund = fund.total_sol_value()?;
        let receipt_token_total_supply = ctx.accounts.receipt_token_mint.supply;
        let receipt_token_price = fund.receipt_token_price(
            ctx.accounts.receipt_token_mint.decimals,
            receipt_token_total_supply,
        )?;

        let mut receipt_token_amount_to_burn: u64 = 0;
        for batch in &mut fund.withdrawal_status.batch_withdrawals_in_progress {
            let amount = batch.receipt_token_to_process;
            batch.record_unstaking_start(amount)?;
            receipt_token_amount_to_burn = receipt_token_amount_to_burn
                .checked_add(amount)
                .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
        }

        let mut receipt_token_amount_not_burned = receipt_token_amount_to_burn;
        let mut total_sol_reserved_amount: u64 = 0;
        for batch in &mut fund.withdrawal_status.batch_withdrawals_in_progress {
            if receipt_token_amount_not_burned == 0 {
                break;
            }

            let receipt_token_amount = std::cmp::min(
                receipt_token_amount_not_burned,
                batch.receipt_token_being_processed,
            );
            receipt_token_amount_not_burned -= receipt_token_amount; // guaranteed to be safe

            let sol_reserved_amount = crate::utils::proportional_amount(
                receipt_token_amount,
                total_sol_value_in_fund,
                receipt_token_total_supply,
            )
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
            total_sol_reserved_amount = total_sol_reserved_amount
                .checked_add(sol_reserved_amount)
                .ok_or_else(|| error!(ErrorCode::CalculationFailure))?;
            batch.record_unstaking_end(receipt_token_amount, sol_reserved_amount)?;
        }
        fund.sol_operation_reserved_amount = fund
            .sol_operation_reserved_amount
            .checked_sub(total_sol_reserved_amount)
            .ok_or_else(|| error!(ErrorCode::FundWithdrawalRequestExceedsSOLAmountsInTemp))?;

        Self::call_burn_token_cpi(&mut ctx, receipt_token_amount_to_burn)?;
        Self::call_transfer_hook(&ctx, receipt_token_amount_to_burn)?;

        ctx.accounts
            .fund
            .withdrawal_status
            .end_processing_completed_batch_withdrawals()?;

        emit!(OperatorRan {
            fund_info: FundInfo::new_from_fund(
                &ctx.accounts.fund,
                receipt_token_price,
                receipt_token_total_supply
            ),
        });

        Ok(())
    }

    fn call_burn_token_cpi(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts.token_program.burn_token_cpi(
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.receipt_token_lock_account,
            ctx.accounts.receipt_token_lock_authority.to_account_info(),
            Some(&[ctx
                .accounts
                .receipt_token_lock_authority
                .signer_seeds()
                .as_ref()]),
            amount,
        )
    }

    fn call_transfer_hook(ctx: &Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts.receipt_token_mint.transfer_hook(
            Some(&ctx.accounts.receipt_token_lock_account),
            None,
            &ctx.accounts.fund,
            amount,
        )
    }
}
