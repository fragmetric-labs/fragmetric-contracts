use anchor_lang::prelude::*;

use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct DenormalizeNTCommand {
    // TODO: DenormalizeNTCommand
}

impl From<DenormalizeNTCommand> for OperationCommand {
    fn from(command: DenormalizeNTCommand) -> Self {
        Self::DenormalizeNT(command)
    }
}

impl SelfExecutable for DenormalizeNTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        _ctx: &mut OperationCommandContext<'info, 'a>,
        _accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        // TODO: DenormalizeNTCommand.execute
        Ok(None)
    }
}
