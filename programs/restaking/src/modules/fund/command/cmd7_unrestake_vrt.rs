use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct UnrestakeVRTCommand {
    // TODO: UnrestakeVRTCommand
}

impl SelfExecutable for UnrestakeVRTCommand {
    fn execute(
        &self,
        _ctx: &mut OperationCommandContext,
        _accounts: &[AccountInfo],
    ) -> Result<Option<OperationCommandEntry>> {
        // TODO: UnrestakeVRTCommand.execute
        Ok(None)
    }
}
