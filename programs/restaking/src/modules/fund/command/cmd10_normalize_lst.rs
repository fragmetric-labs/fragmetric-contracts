use anchor_lang::prelude::*;

use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct NormalizeLSTCommand {
    // TODO: NormalizeLSTCommand
}

impl From<NormalizeLSTCommand> for OperationCommand {
    fn from(command: NormalizeLSTCommand) -> Self {
        Self::NormalizeLST(command)
    }
}

impl SelfExecutable for NormalizeLSTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        _ctx: &mut OperationCommandContext<'info, 'a>,
        _accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        // TODO: NormalizeLSTCommand.execute
        Ok(None)
    }
}
