use anchor_lang::prelude::*;

use super::{
    OperationCommand, OperationCommandContext, OperationCommandEntry, OperationCommandResult,
    SelfExecutable, StakeSOLCommand,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct UndelegateVSTCommand {
    // TODO v0.4.3: UndelegateVSTCommand
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
        Ok((
            None,
            Some(StakeSOLCommand::default().without_required_accounts()),
        ))
    }
}
