use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct DenormalizeNTCommand {
    // TODO: DenormalizeNTCommand
}

impl SelfExecutable for DenormalizeNTCommand {
    fn execute(
        &self,
        ctx: &mut OperationCommandContext,
        accounts: &[AccountInfo],
    ) -> Result<Vec<OperationCommandEntry>> {
        // TODO: DenormalizeNTCommand.execute
        Ok(vec![])
    }
}
