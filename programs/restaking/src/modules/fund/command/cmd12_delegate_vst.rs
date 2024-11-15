use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct DelegateVSTCommand {
    // TODO: DelegateVSTCommand
}

impl SelfExecutable for DelegateVSTCommand {
    fn execute(
        &self,
        ctx: &mut OperationCommandContext,
        accounts: &[AccountInfo],
    ) -> Result<Vec<OperationCommandEntry>> {
        // TODO: DelegateVSTCommand.execute
        Ok(vec![])
    }
}
