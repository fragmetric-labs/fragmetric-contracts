use anchor_lang::prelude::*;

use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ClaimUnstakedSOLCommand {
    // TODO: ClaimUnstakedSOLCommand
}

impl From<ClaimUnstakedSOLCommand> for OperationCommand {
    fn from(command: ClaimUnstakedSOLCommand) -> Self {
        Self::ClaimUnstakedSOL(command)
    }
}

impl SelfExecutable for ClaimUnstakedSOLCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        _ctx: &mut OperationCommandContext<'info, 'a>,
        _accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        // TODO: ClaimUnstakedSOLCommand.execute
        Ok(None)
    }
}
