use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ClaimUnrestakedVSTCommand {
    // TODO: ClaimUnrestakedVSTCommand
}

impl SelfExecutable for ClaimUnrestakedVSTCommand {
    fn execute(
        &self,
        _ctx: &mut OperationCommandContext,
        _accounts: &[AccountInfo],
    ) -> Result<Option<OperationCommandEntry>> {
        // TODO: ClaimUnrestakedVSTCommand.execute
        Ok(None)
    }
}
