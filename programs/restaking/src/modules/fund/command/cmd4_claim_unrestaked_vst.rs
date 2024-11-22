use anchor_lang::prelude::*;

use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ClaimUnrestakedVSTCommand {
    // TODO: ClaimUnrestakedVSTCommand
}

impl From<ClaimUnrestakedVSTCommand> for OperationCommand {
    fn from(command: ClaimUnrestakedVSTCommand) -> Self {
        Self::ClaimUnrestakedVST(command)
    }
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
