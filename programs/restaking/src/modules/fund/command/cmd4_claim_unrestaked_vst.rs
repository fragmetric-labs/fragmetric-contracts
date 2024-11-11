use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct ClaimUnrestakedVSTCommand {
    // TODO: ClaimUnrestakedVSTCommand
}

impl SelfExecutable for ClaimUnrestakedVSTCommand {
    fn execute(
        &self,
        context: &OperationCommandContext,
        accounts: Vec<&AccountInfo>,
    ) -> Result<Vec<OperationCommandEntry>> {
        // TODO: ClaimUnrestakedVSTCommand.execute
        Ok(vec![])
    }

    fn compute_required_accounts(&self, context: &OperationCommandContext) -> Result<Vec<Pubkey>> {
        // TODO: ClaimUnrestakedVSTCommand.compute_required_accounts
        Ok(vec![])
    }
}
