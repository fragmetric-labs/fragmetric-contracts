use anchor_lang::prelude::*;

use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, OperationCommandResult, SelfExecutable, UnrestakeVRTCommand};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct UndelegateVSTCommand {
    // TODO: UndelegateVSTCommand
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct UndelegateVSTCommandResult {}

impl SelfExecutable for UndelegateVSTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        _ctx: &mut OperationCommandContext<'info, 'a>,
        _accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        // TODO: UndelegateVSTCommand.execute
        Ok((None, Some(UnrestakeVRTCommand::default().without_required_accounts())))
    }
}
