use anchor_lang::prelude::*;

use crate::modules::fund;

use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct ProcessWithdrawalBatchCommand {
    state: ProcessWithdrawalBatchCommandState,
    forced: bool,
}

impl From<ProcessWithdrawalBatchCommand> for OperationCommand {
    fn from(command: ProcessWithdrawalBatchCommand) -> Self {
        Self::ProcessWithdrawalBatch(command)
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum ProcessWithdrawalBatchCommandState {
    #[default]
    Init,
    Process,
}

impl SelfExecutable for ProcessWithdrawalBatchCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        match self.state {
            ProcessWithdrawalBatchCommandState::Init => {
                let mut command = self.clone();
                command.state = ProcessWithdrawalBatchCommandState::Process;

                let required_accounts =
                    fund::FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                        .find_accounts_to_process_withdrawal_batch()?;
                return Ok(Some(command.with_required_accounts(required_accounts)));
            }
            ProcessWithdrawalBatchCommandState::Process => {
                let [system_program, receipt_token_program, receipt_token_lock_account, fund_reserve_account, treasury_account, remaining_accounts @ ..] =
                    accounts
                else {
                    err!(ErrorCode::AccountNotEnoughKeys)?
                };

                if remaining_accounts.len() < ctx.fund_account.withdrawal.queued_batches.len() {
                    err!(ErrorCode::AccountNotEnoughKeys)?;
                }

                let (uninitialized_batch_withdrawal_tickets, pricing_sources) =
                    remaining_accounts.split_at(ctx.fund_account.withdrawal.queued_batches.len());
                fund::FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                    .process_withdrawal_batch(
                        ctx.operator,
                        system_program,
                        receipt_token_program,
                        receipt_token_lock_account,
                        fund_reserve_account,
                        treasury_account,
                        uninitialized_batch_withdrawal_tickets,
                        pricing_sources,
                        self.forced,
                    )?;
            }
        }
        // TODO: ProcessWithdrawalBatchCommand.execute
        Ok(None)
    }
}
