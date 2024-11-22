use anchor_lang::prelude::*;

use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct RestakeVSTCommand {
    // TODO: RestakeVSTCommand
}

impl From<RestakeVSTCommand> for OperationCommand {
    fn from(command: RestakeVSTCommand) -> Self {
        Self::RestakeVST(command)
    }
}

impl SelfExecutable for RestakeVSTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        _ctx: &mut OperationCommandContext<'info, 'a>,
        _accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        // TODO: RestakeVSTCommand.execute
        Ok(None)
    }
}
