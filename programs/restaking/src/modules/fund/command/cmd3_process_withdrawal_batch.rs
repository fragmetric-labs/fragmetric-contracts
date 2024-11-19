use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ProcessWithdrawalBatchCommand {
    // TODO: ProcessWithdrawalBatchCommand
}

impl SelfExecutable for ProcessWithdrawalBatchCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        _ctx: &mut OperationCommandContext<'info, 'a>,
        _accounts: &'a [AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        // TODO: ProcessWithdrawalBatchCommand.execute
        Ok(None)
    }
}
