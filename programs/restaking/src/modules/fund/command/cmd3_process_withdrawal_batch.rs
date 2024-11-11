use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct ProcessWithdrawalBatchCommand {
    // TODO: ProcessWithdrawalBatchCommand
}

impl SelfExecutable for ProcessWithdrawalBatchCommand {
    fn execute(
        &self,
        context: &OperationCommandContext,
        accounts: Vec<&AccountInfo>,
    ) -> Result<Vec<OperationCommandEntry>> {
        // TODO: ProcessWithdrawalBatchCommand.execute
        Ok(vec![])
    }

    fn compute_required_accounts(&self, context: &OperationCommandContext) -> Result<Vec<Pubkey>> {
        // TODO: ProcessWithdrawalBatchCommand.compute_required_accounts
        Ok(vec![])
    }
}
