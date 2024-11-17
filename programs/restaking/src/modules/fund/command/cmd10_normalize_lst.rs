use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct NormalizeLSTCommand {
    // TODO: NormalizeLSTCommand
}

impl SelfExecutable for NormalizeLSTCommand {
    fn execute(
        &self,
        _ctx: &mut OperationCommandContext,
        _accounts: &[AccountInfo],
    ) -> Result<Option<OperationCommandEntry>> {
        // TODO: NormalizeLSTCommand.execute
        Ok(None)
    }
}
