use anchor_lang::prelude::*;

use crate::modules::fund::FundService;
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
    New,
    /// max receipt_token_amount to process withdrawal
    Process(u64),
}

impl SelfExecutable for ProcessWithdrawalBatchCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        match self.state {
            ProcessWithdrawalBatchCommandState::New => {
                let mut fund_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?;
                let pricing_service = fund_service.new_pricing_service(accounts.into_iter().cloned())?;

                let receipt_token_amount_to_process = fund_service.get_sol_withdrawal_execution_amount(&pricing_service)?;

                let mut command = self.clone();
                command.state = ProcessWithdrawalBatchCommandState::Process(receipt_token_amount_to_process);

                let required_accounts = fund_service.find_accounts_to_process_withdrawal_batch()?;
                return Ok(Some(command.with_required_accounts(required_accounts)));
            }
            ProcessWithdrawalBatchCommandState::Process(receipt_token_amount_to_process) => {
                let [receipt_token_program, receipt_token_lock_account, fund_reserve_account, treasury_account, remaining_accounts @ ..] =
                    accounts
                else {
                    err!(ErrorCode::AccountNotEnoughKeys)?
                };

                let num_queued_batches = ctx.fund_account.load()?.get_withdrawal_state().get_queued_batches_iter().count();
                if remaining_accounts.len() < num_queued_batches {
                    err!(ErrorCode::AccountNotEnoughKeys)?;
                }

                let (uninitialized_batch_withdrawal_tickets, pricing_sources) =
                    remaining_accounts.split_at(num_queued_batches);

                FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                    .process_withdrawal_batch(
                        ctx.operator,
                        ctx.system_program,
                        receipt_token_program,
                        receipt_token_lock_account,
                        fund_reserve_account,
                        treasury_account,
                        uninitialized_batch_withdrawal_tickets,
                        pricing_sources,
                        self.forced,
                        receipt_token_amount_to_process,
                    )?;
            }
        }
        // TODO: ProcessWithdrawalBatchCommand.execute
        Ok(None)
    }
}
