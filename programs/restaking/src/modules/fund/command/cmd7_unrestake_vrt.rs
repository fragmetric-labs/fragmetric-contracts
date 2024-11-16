use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct UnrestakeVRTCommand {
    // TODO: UnrestakeVRTCommand
}

impl SelfExecutable for UnrestakeVRTCommand {
    fn execute(
        &self,
        ctx: &mut OperationCommandContext,
        accounts: &[AccountInfo],
    ) -> Result<Vec<OperationCommandEntry>> {
        // TODO: UnrestakeVRTCommand.execute
        Ok(vec![])
    }
}
