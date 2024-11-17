use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};
use crate::modules::fund;
use crate::modules::fund::FundService;
use anchor_lang::prelude::*;
use anchor_spl::token_2022;
use crate::utils::PDASeeds;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct EnqueueWithdrawalBatchCommand {
    state: EnqueueWithdrawalBatchCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum EnqueueWithdrawalBatchCommandState {
    Init,
    Enqueue,
}

impl SelfExecutable for EnqueueWithdrawalBatchCommand {
    fn execute(
        &self,
        ctx: &mut OperationCommandContext,
        accounts: &[AccountInfo],
    ) -> Result<Option<OperationCommandEntry>> {
        match self.state {
            EnqueueWithdrawalBatchCommandState::Init => {
                let mut command = self.clone();
                command.state = EnqueueWithdrawalBatchCommandState::Enqueue;

                return Ok(Some(
                    OperationCommand::EnqueueWithdrawalBatch(command).with_required_accounts(vec![
                        ctx.fund_account.key(),
                        ctx.fund_account.find_receipt_token_program_address(),
                        ctx.fund_account
                            .find_receipt_token_lock_account_address()?
                            .0,
                    ]),
                ));
            }
            EnqueueWithdrawalBatchCommandState::Enqueue => {
                let [receipt_token_program, receipt_token_lock_account, remaining_accounts @ ..] =
                    accounts
                else {
                    err!(ErrorCode::AccountNotEnoughKeys)?
                };

                let mut withdrawal_state = std::mem::take(&mut ctx.fund_account.withdrawal);
                let current_timestamp = Clock::get()?.unix_timestamp;

                withdrawal_state.assert_withdrawal_threshold_satisfied(current_timestamp)?;
                withdrawal_state
                    .start_processing_pending_batch_withdrawal(current_timestamp)?;

                let pricing_service = FundService::new(&mut ctx.receipt_token_mint.clone(), &mut ctx.fund_account.clone())?
                    .new_pricing_service(remaining_accounts)?;

                let mut receipt_token_amount_to_burn: u64 = 0;
                for batch in &mut withdrawal_state.batch_withdrawals_in_progress {
                    let amount = batch.receipt_token_to_process;
                    batch.record_unstaking_start(amount)?;
                    receipt_token_amount_to_burn = receipt_token_amount_to_burn
                        .checked_add(amount)
                        .ok_or_else(|| error!(crate::errors::ErrorCode::CalculationArithmeticException))?;
                }

                let mut receipt_token_amount_not_burned = receipt_token_amount_to_burn;
                let mut total_sol_reserved_amount: u64 = 0;
                for batch in &mut withdrawal_state.batch_withdrawals_in_progress {
                    if receipt_token_amount_not_burned == 0 {
                        break;
                    }

                    let receipt_token_amount = std::cmp::min(
                        receipt_token_amount_not_burned,
                        batch.receipt_token_being_processed,
                    );
                    receipt_token_amount_not_burned -= receipt_token_amount; // guaranteed to be safe

                    let sol_reserved_amount = pricing_service.get_token_amount_as_sol(
                        &ctx.receipt_token_mint.key(),
                        receipt_token_amount,
                    )?;
                    total_sol_reserved_amount = total_sol_reserved_amount
                        .checked_add(sol_reserved_amount)
                        .ok_or_else(|| error!(crate::errors::ErrorCode::CalculationArithmeticException))?;
                    batch.record_unstaking_end(receipt_token_amount, sol_reserved_amount)?;
                }
                ctx.fund_account.sol_operation_reserved_amount = ctx
                    .fund_account
                    .sol_operation_reserved_amount
                    .checked_sub(total_sol_reserved_amount)
                    .ok_or_else(|| error!(crate::errors::ErrorCode::FundOperationReservedSOLExhaustedException))?;

                token_2022::burn(
                    CpiContext::new_with_signer(
                        receipt_token_program.to_account_info(),
                        token_2022::Burn {
                            mint: ctx.receipt_token_mint.to_account_info(),
                            from: receipt_token_lock_account.to_account_info(),
                            authority: ctx.fund_account.to_account_info(),
                        },
                        &[ctx.fund_account.get_signer_seeds().as_ref()],
                    ),
                    receipt_token_amount_to_burn,
                )?;
                ctx.receipt_token_mint.reload()?;
                // TODO: receipt_token_lock_account.reload()?;

                withdrawal_state
                    .end_processing_completed_batch_withdrawals(current_timestamp)?;

                // write back operation state
                ctx.fund_account.withdrawal = withdrawal_state;
            }
        }

        Ok(None)
    }
}
