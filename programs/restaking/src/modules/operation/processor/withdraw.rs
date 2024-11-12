use anchor_lang::prelude::*;
use anchor_spl::token_2022;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::constants::ADMIN_PUBKEY;
use crate::errors::ErrorCode;
use crate::events;
use crate::modules::fund::{self, FundAccount, FundAccountInfo};
use crate::utils::PDASeeds;

// TODO: deprecate
pub fn process_process_fund_withdrawal_job<'info>(
    operator: &Signer,
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    receipt_token_lock_account: &mut InterfaceAccount<'info, TokenAccount>,
    fund_account: &mut Account<'info, FundAccount>,
    receipt_token_program: &Program<'info, Token2022>,
    pricing_sources: &'info [AccountInfo<'info>],
    forced: bool,
    current_timestamp: i64,
) -> Result<()> {
    if !(forced && operator.key() == ADMIN_PUBKEY) {
        fund_account
            .withdrawal
            .assert_withdrawal_threshold_satisfied(current_timestamp)?;
    }

    fund::update_asset_prices(fund_account, pricing_sources)?;

    fund_account
        .withdrawal
        .start_processing_pending_batch_withdrawal(current_timestamp)?;

    let assets_total_amount_as_sol = fund_account.get_assets_total_amount_as_sol()?;
    let mut receipt_token_amount_to_burn: u64 = 0;
    for batch in &mut fund_account.withdrawal.batch_withdrawals_in_progress {
        let amount = batch.receipt_token_to_process;
        batch.record_unstaking_start(amount)?;
        receipt_token_amount_to_burn = receipt_token_amount_to_burn
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
    }

    let mut receipt_token_amount_not_burned = receipt_token_amount_to_burn;
    let mut total_sol_reserved_amount: u64 = 0;
    for batch in &mut fund_account.withdrawal.batch_withdrawals_in_progress {
        if receipt_token_amount_not_burned == 0 {
            break;
        }

        let receipt_token_amount = std::cmp::min(
            receipt_token_amount_not_burned,
            batch.receipt_token_being_processed,
        );
        receipt_token_amount_not_burned -= receipt_token_amount; // guaranteed to be safe

        let sol_reserved_amount = crate::utils::get_proportional_amount(
            receipt_token_amount,
            assets_total_amount_as_sol,
            receipt_token_mint.supply,
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
                authority: fund_account.to_account_info(),
            },
            &[fund_account.get_signer_seeds().as_ref()],
        ),
        receipt_token_amount_to_burn,
    )?;
    receipt_token_mint.reload()?;
    receipt_token_lock_account.reload()?;

    fund_account
        .withdrawal
        .end_processing_completed_batch_withdrawals(current_timestamp)?;

    emit!(events::OperatorProcessedJob {
        receipt_token_mint: receipt_token_mint.key(),
        fund_account: FundAccountInfo::from(
            fund_account,
            receipt_token_mint,
        ),
    });

    Ok(())
}
