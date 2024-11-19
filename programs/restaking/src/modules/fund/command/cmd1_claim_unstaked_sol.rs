use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ClaimUnstakedSOLCommand {
    // TODO: ClaimUnstakedSOLCommand
}

impl SelfExecutable for ClaimUnstakedSOLCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        _ctx: &mut OperationCommandContext<'info, 'a>,
        _accounts: &'a [AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        // TODO: ClaimUnstakedSOLCommand.execute
        Ok(None)
    }
}
