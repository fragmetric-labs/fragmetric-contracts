use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct UndelegateVSTCommand {
    // TODO: UndelegateVSTCommand
}

impl SelfExecutable for UndelegateVSTCommand {
    fn execute(
        &self,
        ctx: &mut OperationCommandContext,
        accounts: &[AccountInfo],
    ) -> Result<Vec<OperationCommandEntry>> {
        // TODO: UndelegateVSTCommand.execute
        Ok(vec![])
    }
}
