use super::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct UnrestakeVRTCommand {
    // TODO: UnrestakeVRTCommand
}

impl SelfExecutable for UnrestakeVRTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        _ctx: &mut OperationCommandContext<'info, 'a>,
        _accounts: &'a [AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        // TODO: UnrestakeVRTCommand.execute
        Ok(None)
    }
}
