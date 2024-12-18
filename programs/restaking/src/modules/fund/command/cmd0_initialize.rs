use anchor_lang::prelude::*;

use super::cmd9_stake_sol::StakeSOLCommand;
use super::{
    OperationCommand, OperationCommandContext, OperationCommandEntry, OperationCommandResult,
    SelfExecutable,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct InitializeCommand {}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct InitializeCommandResult {}

impl SelfExecutable for InitializeCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        _accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        // TODO v0.3/operation: proceed to claim_unstaked_sol command
        Ok((
            None,
            Some(StakeSOLCommand::default().without_required_accounts()),
        ))
    }
}
