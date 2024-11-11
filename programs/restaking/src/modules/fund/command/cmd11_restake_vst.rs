use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct RestakeVSTCommand {
    // TODO: RestakeVSTCommand
}

impl SelfExecutable for RestakeVSTCommand {
    fn execute(
        &self,
        context: &OperationCommandContext,
        accounts: Vec<&AccountInfo>,
    ) -> Result<Vec<OperationCommandEntry>> {
        // TODO: RestakeVSTCommand.execute
        Ok(vec![])
    }

    fn compute_required_accounts(&self, context: &OperationCommandContext) -> Result<Vec<Pubkey>> {
        // TODO: RestakeVSTCommand.compute_required_accounts
        Ok(vec![])
    }
}
