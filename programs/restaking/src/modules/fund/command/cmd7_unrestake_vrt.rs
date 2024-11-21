use anchor_lang::prelude::*;

use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct UnrestakeVRTCommand {
    // TODO: UnrestakeVRTCommand
}

impl From<UnrestakeVRTCommand> for OperationCommand {
    fn from(command: UnrestakeVRTCommand) -> Self {
        Self::UnrestakeVRT(command)
    }
}

impl SelfExecutable for UnrestakeVRTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        _ctx: &mut OperationCommandContext<'info, 'a>,
        _accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        // TODO: UnrestakeVRTCommand.execute
        Ok(None)
    }
}
