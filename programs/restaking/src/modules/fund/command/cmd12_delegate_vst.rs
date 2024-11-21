use anchor_lang::prelude::*;

use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct DelegateVSTCommand {
    // TODO: DelegateVSTCommand
}

impl From<DelegateVSTCommand> for OperationCommand {
    fn from(command: DelegateVSTCommand) -> Self {
        Self::DelegateVST(command)
    }
}

impl SelfExecutable for DelegateVSTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        _ctx: &mut OperationCommandContext<'info, 'a>,
        _accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        // TODO: DelegateVSTCommand.execute
        Ok(None)
    }
}
