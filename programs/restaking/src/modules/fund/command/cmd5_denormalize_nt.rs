use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct DenormalizeNTCommand {
    // TODO: DenormalizeNTCommand
}

impl SelfExecutable for DenormalizeNTCommand {
    fn execute(
        &self,
        _ctx: &mut OperationCommandContext,
        _accounts: &[AccountInfo],
    ) -> Result<Option<OperationCommandEntry>> {
        // TODO: DenormalizeNTCommand.execute
        Ok(None)
    }
}
