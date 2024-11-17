use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct DelegateVSTCommand {
    // TODO: DelegateVSTCommand
}

impl SelfExecutable for DelegateVSTCommand {
    fn execute(
        &self,
        _ctx: &mut OperationCommandContext,
        _accounts: &[AccountInfo],
    ) -> Result<Option<OperationCommandEntry>> {
        // TODO: DelegateVSTCommand.execute
        Ok(None)
    }
}
