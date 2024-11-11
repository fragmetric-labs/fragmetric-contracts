use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct EnqueueWithdrawalBatchCommand {
    // TODO: EnqueueWithdrawalBatchCommand
}

impl SelfExecutable for EnqueueWithdrawalBatchCommand {
    fn execute(
        &self,
        context: &OperationCommandContext,
        accounts: Vec<&AccountInfo>,
    ) -> Result<Vec<OperationCommandEntry>> {
        // TODO: EnqueueWithdrawalBatchCommand.execute
        Ok(vec![])
    }

    fn compute_required_accounts(&self, context: &OperationCommandContext) -> Result<Vec<Pubkey>> {
        // TODO: EnqueueWithdrawalBatchCommand.compute_required_accounts
        Ok(vec![])
    }
}
