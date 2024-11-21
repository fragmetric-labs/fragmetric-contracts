use anchor_lang::prelude::*;

use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct UndelegateVSTCommand {
    // TODO: UndelegateVSTCommand
}

impl From<UndelegateVSTCommand> for OperationCommand {
    fn from(command: UndelegateVSTCommand) -> Self {
        Self::UndelegateVST(command)
    }
}

impl SelfExecutable for UndelegateVSTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        _ctx: &mut OperationCommandContext<'info, 'a>,
        _accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        // TODO: UndelegateVSTCommand.execute
        Ok(None)
    }
}
