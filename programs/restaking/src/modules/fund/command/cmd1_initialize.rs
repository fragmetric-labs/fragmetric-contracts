use anchor_lang::prelude::*;

use super::cmd10_stake_sol::StakeSOLCommand;
use super::{
    EnqueueWithdrawalBatchCommand, OperationCommand, OperationCommandContext,
    OperationCommandEntry, OperationCommandResult, SelfExecutable,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct InitializeCommand {}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct InitializeCommandResult {}

impl SelfExecutable for InitializeCommand {
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
            Some(EnqueueWithdrawalBatchCommand::default().without_required_accounts()),
        ))
    }
}
