use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct RestakeVSTCommand {
    // TODO: RestakeVSTCommand
}

impl SelfExecutable for RestakeVSTCommand {
    fn execute(
        &self,
        ctx: &mut OperationCommandContext,
        accounts: &[AccountInfo],
    ) -> Result<Vec<OperationCommandEntry>> {
        // TODO: RestakeVSTCommand.execute
        Ok(vec![])
    }
}
