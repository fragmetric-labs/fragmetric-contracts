use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct UnstakeLSTCommand {
    // TODO: UnstakeLSTCommand
}

impl SelfExecutable for UnstakeLSTCommand {
    fn execute(
        &self,
        ctx: &mut OperationCommandContext,
        accounts: &[AccountInfo],
    ) -> Result<Vec<OperationCommandEntry>> {
        // TODO: UnstakeLSTCommand.execute
        Ok(vec![])
    }
}
