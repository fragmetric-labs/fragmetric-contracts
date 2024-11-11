use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct DenormalizeNTCommand {
    // TODO: DenormalizeNTCommand
}

impl SelfExecutable for DenormalizeNTCommand {
    fn execute(
        &self,
        context: &OperationCommandContext,
        accounts: Vec<&AccountInfo>,
    ) -> Result<Vec<OperationCommandEntry>> {
        // TODO: DenormalizeNTCommand.execute
        Ok(vec![])
    }

    fn compute_required_accounts(&self, context: &OperationCommandContext) -> Result<Vec<Pubkey>> {
        // TODO: DenormalizeNTCommand.compute_required_accounts
        Ok(vec![])
    }
}
