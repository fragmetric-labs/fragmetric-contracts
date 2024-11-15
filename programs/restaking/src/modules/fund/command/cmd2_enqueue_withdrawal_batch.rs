use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct EnqueueWithdrawalBatchCommand {
    // TODO: EnqueueWithdrawalBatchCommand
}

impl SelfExecutable for EnqueueWithdrawalBatchCommand {
    fn execute(
        &self,
        ctx: &mut OperationCommandContext,
        accounts: &[AccountInfo],
    ) -> Result<Vec<OperationCommandEntry>> {
        // TODO: EnqueueWithdrawalBatchCommand.execute
        Ok(vec![])
    }
}
