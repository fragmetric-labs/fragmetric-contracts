use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct ClaimUnrestakedVSTCommand {
    // TODO: ClaimUnrestakedVSTCommand
}

impl SelfExecutable for ClaimUnrestakedVSTCommand {
    fn execute(
        &self,
        ctx: &mut OperationCommandContext,
        accounts: &[AccountInfo],
    ) -> Result<Vec<OperationCommandEntry>> {
        // TODO: ClaimUnrestakedVSTCommand.execute
        Ok(vec![])
    }
}
