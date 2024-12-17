use anchor_lang::prelude::*;

use crate::modules::fund;

use super::cmd3_process_withdrawal_batch::ProcessWithdrawalBatchCommand;
use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct EnqueueWithdrawalBatchCommand {
    forced: bool,
}

impl From<EnqueueWithdrawalBatchCommand> for OperationCommand {
    fn from(command: EnqueueWithdrawalBatchCommand) -> Self {
        Self::EnqueueWithdrawalBatch(command)
    }
}

impl SelfExecutable for EnqueueWithdrawalBatchCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        _accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        fund::FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
            .enqueue_withdrawal_batches(self.forced)?;

        Ok(Some(
            ProcessWithdrawalBatchCommand::default().without_required_accounts(),
        ))
    }
}
