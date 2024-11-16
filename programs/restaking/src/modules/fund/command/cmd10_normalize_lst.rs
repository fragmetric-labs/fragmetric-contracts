use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct NormalizeLSTCommand {
    // TODO: NormalizeLSTCommand
}

impl SelfExecutable for NormalizeLSTCommand {
    fn execute(
        &self,
        ctx: &mut OperationCommandContext,
        accounts: &[AccountInfo],
    ) -> Result<Vec<OperationCommandEntry>> {
        // TODO: NormalizeLSTCommand.execute
        Ok(vec![])
    }
}
