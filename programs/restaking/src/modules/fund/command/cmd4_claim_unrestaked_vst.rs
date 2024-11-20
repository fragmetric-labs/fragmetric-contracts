use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ClaimUnrestakedVSTCommand {
    // TODO: ClaimUnrestakedVSTCommand
}

impl SelfExecutable for ClaimUnrestakedVSTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        _ctx: &mut OperationCommandContext<'info, 'a>,
        _accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        // TODO: ClaimUnrestakedVSTCommand.execute
        Ok(None)
    }
}
