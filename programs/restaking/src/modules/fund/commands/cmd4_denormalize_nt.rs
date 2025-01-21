use anchor_lang::prelude::*;
use crate::modules::fund::commands::OperationCommand::UndelegateVST;
use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, OperationCommandResult, SelfExecutable, UndelegateVSTCommand};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct DenormalizeNTCommand {
    // TODO: DenormalizeNTCommand
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct DenormalizeNTCommandResult {}

impl SelfExecutable for DenormalizeNTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        _ctx: &mut OperationCommandContext<'info, 'a>,
        _accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        // TODO: DenormalizeNTCommand.execute
        Ok((None, Some(UndelegateVSTCommand::default().without_required_accounts())))
    }
}
