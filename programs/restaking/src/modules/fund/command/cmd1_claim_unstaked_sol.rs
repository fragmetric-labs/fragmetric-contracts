use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct ClaimUnstakedSOLCommand {
    // TODO: ClaimUnstakedSOLCommand
}

impl SelfExecutable for ClaimUnstakedSOLCommand {
    fn execute(
        &self,
        context: &OperationCommandContext,
        accounts: Vec<&AccountInfo>,
    ) -> Result<Vec<OperationCommandEntry>> {
        // TODO: ClaimUnstakedSOLCommand.execute
        Ok(vec![])
    }

    fn compute_required_accounts(&self, context: &OperationCommandContext) -> Result<Vec<Pubkey>> {
        // TODO: ClaimUnstakedSOLCommand.compute_required_accounts
        Ok(vec![])
    }
}
