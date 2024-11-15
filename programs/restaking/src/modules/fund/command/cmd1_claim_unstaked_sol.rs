use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct ClaimUnstakedSOLCommand {
    // TODO: ClaimUnstakedSOLCommand
}

impl SelfExecutable for ClaimUnstakedSOLCommand {
    fn execute(
        &self,
        ctx: &mut OperationCommandContext,
        accounts: &[AccountInfo],
    ) -> Result<Vec<OperationCommandEntry>> {
        // TODO: ClaimUnstakedSOLCommand.execute
        Ok(vec![])
    }
}
