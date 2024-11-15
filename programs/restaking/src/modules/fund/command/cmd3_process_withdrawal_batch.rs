use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct ProcessWithdrawalBatchCommand {
    // TODO: ProcessWithdrawalBatchCommand
}

impl SelfExecutable for ProcessWithdrawalBatchCommand {
    fn execute(
        &self,
        ctx: &mut OperationCommandContext,
        accounts: &[AccountInfo],
    ) -> Result<Vec<OperationCommandEntry>> {
        // TODO: ProcessWithdrawalBatchCommand.execute
        Ok(vec![])
    }
}
