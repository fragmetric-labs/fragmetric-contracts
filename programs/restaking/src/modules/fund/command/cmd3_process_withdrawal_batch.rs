use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ProcessWithdrawalBatchCommand {
    // TODO: ProcessWithdrawalBatchCommand
}

impl SelfExecutable for ProcessWithdrawalBatchCommand {
    fn execute(
        &self,
        _ctx: &mut OperationCommandContext,
        _accounts: &[AccountInfo],
    ) -> Result<Option<OperationCommandEntry>> {
        // TODO: ProcessWithdrawalBatchCommand.execute
        Ok(None)
    }
}
