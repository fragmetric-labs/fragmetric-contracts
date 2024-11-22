use anchor_lang::prelude::*;

use crate::modules::fund;

use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct EnqueueWithdrawalBatchCommand {
    state: EnqueueWithdrawalBatchCommandState,
    forced: bool,
}

impl From<EnqueueWithdrawalBatchCommand> for OperationCommand {
    fn from(command: EnqueueWithdrawalBatchCommand) -> Self {
        Self::EnqueueWithdrawalBatch(command)
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum EnqueueWithdrawalBatchCommandState {
    #[default]
    Init,
    Enqueue,
}

impl SelfExecutable for EnqueueWithdrawalBatchCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        match self.state {
            EnqueueWithdrawalBatchCommandState::Init => {
                let mut command = self.clone();
                command.state = EnqueueWithdrawalBatchCommandState::Enqueue;

                return Ok(Some(command.with_required_accounts([
                    (ctx.fund_account.receipt_token_program, false),
                    (
                        ctx.fund_account.find_receipt_token_lock_account_address()?,
                        true,
                    ),
                ])));
            }
            EnqueueWithdrawalBatchCommandState::Enqueue => {
                let [receipt_token_program, receipt_token_lock_account, remaining_accounts @ ..] =
                    accounts
                else {
                    err!(ErrorCode::AccountNotEnoughKeys)?
                };

                fund::FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                    .enqueue_withdrawal_batch(
                        receipt_token_program,
                        receipt_token_lock_account,
                        remaining_accounts.iter().cloned(),
                        self.forced,
                    )?;
            }
        }

        Ok(None)
    }
}
