use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct UndelegateVSTCommand {
    // TODO: UndelegateVSTCommand
}

impl SelfExecutable for UndelegateVSTCommand {
    fn execute(
        &self,
        _ctx: &mut OperationCommandContext,
        _accounts: &[AccountInfo],
    ) -> Result<Option<OperationCommandEntry>> {
        // TODO: UndelegateVSTCommand.execute
        Ok(None)
    }
}
