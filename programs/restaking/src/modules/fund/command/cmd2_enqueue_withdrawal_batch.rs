use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct EnqueueWithdrawalBatchCommand {
    // TODO: EnqueueWithdrawalBatchCommand
}

impl SelfExecutable for EnqueueWithdrawalBatchCommand {
    fn execute(
        &self,
        _ctx: &mut OperationCommandContext,
        _accounts: &[AccountInfo],
    ) -> Result<Option<OperationCommandEntry>> {
        // TODO: EnqueueWithdrawalBatchCommand.execute
        Ok(None)
    }
}
