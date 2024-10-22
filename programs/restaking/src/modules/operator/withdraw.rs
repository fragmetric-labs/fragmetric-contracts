use anchor_lang::prelude::*;
use anchor_spl::token_2022;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::constants::ADMIN_PUBKEY;
use crate::errors::ErrorCode;
use crate::events;
use crate::modules::fund::{FundAccount, FundAccountInfo, ReceiptTokenLockAuthority};
use crate::utils::PDASeeds;

pub fn process_process_fund_withdrawal_job<'info>(
    operator: &Signer,
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    receipt_token_lock_account: &mut InterfaceAccount<'info, TokenAccount>,
    receipt_token_lock_authority: &Account<'info, ReceiptTokenLockAuthority>,
    fund_account: &mut Account<'info, FundAccount>,
    receipt_token_program: &Program<'info, Token2022>,
    pricing_sources: &'info [AccountInfo<'info>],
    forced: bool,
    current_time: i64,
) -> Result<()> {
    if !(forced && operator.key() == ADMIN_PUBKEY) {
        fund_account
            .withdrawal_status
            .check_withdrawal_threshold(current_time)?;
    }

    fund_account.update_token_prices(pricing_sources)?;

    fund_account
        .withdrawal_status
        .start_processing_pending_batch_withdrawal()?;

    let total_sol_value_in_fund = fund_account.assets_total_sol_value()?;
    let receipt_token_total_supply = receipt_token_mint.supply;

    let mut receipt_token_amount_to_burn: u64 = 0;
    for batch in &mut fund_account.withdrawal_status.batch_withdrawals_in_progress {
        let amount = batch.receipt_token_to_process;
        batch.record_unstaking_start(amount)?;
        receipt_token_amount_to_burn = receipt_token_amount_to_burn
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
    }

    let mut receipt_token_amount_not_burned = receipt_token_amount_to_burn;
    let mut total_sol_reserved_amount: u64 = 0;
    for batch in &mut fund_account.withdrawal_status.batch_withdrawals_in_progress {
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
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        total_sol_reserved_amount = total_sol_reserved_amount
            .checked_add(sol_reserved_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        batch.record_unstaking_end(receipt_token_amount, sol_reserved_amount)?;
    }
    fund_account.sol_operation_reserved_amount = fund_account
        .sol_operation_reserved_amount
        .checked_sub(total_sol_reserved_amount)
        .ok_or_else(|| error!(ErrorCode::FundOperationReservedSOLExhaustedException))?;

    token_2022::burn(
        CpiContext::new_with_signer(
            receipt_token_program.to_account_info(),
            token_2022::Burn {
                mint: receipt_token_mint.to_account_info(),
                from: receipt_token_lock_account.to_account_info(),
                authority: receipt_token_lock_authority.to_account_info(),
            },
            &[receipt_token_lock_authority.signer_seeds().as_ref()],
        ),
        receipt_token_amount_to_burn,
    )?;
    receipt_token_mint.reload()?;
    receipt_token_lock_account.reload()?;

    fund_account
        .withdrawal_status
        .end_processing_completed_batch_withdrawals()?;

    let receipt_token_price = fund_account.receipt_token_sol_value_per_token(
        receipt_token_mint.decimals,
        receipt_token_mint.supply,
    )?;

    emit!(events::OperatorProcessedJob {
        receipt_token_mint: receipt_token_mint.key(),
        fund_account: FundAccountInfo::new(
            fund_account,
            receipt_token_price,
            receipt_token_mint.supply,
        ),
    });

    Ok(())
}
