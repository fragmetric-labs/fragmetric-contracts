use anchor_lang::prelude::*;

use crate::modules::fund;

use super::cmd3_process_withdrawal_batch::ProcessWithdrawalBatchCommand;
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
        _accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        match self.state {
            EnqueueWithdrawalBatchCommandState::Init => {
                let mut command = self.clone();
                command.state = EnqueueWithdrawalBatchCommandState::Enqueue;

                return Ok(Some(command.with_required_accounts([])));
            }
            EnqueueWithdrawalBatchCommandState::Enqueue => {
                fund::FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                    .enqueue_withdrawal_batch(self.forced)?;
            }
        }

        Ok(Some(
            ProcessWithdrawalBatchCommand::default().with_required_accounts([]),
        ))
    }
}
