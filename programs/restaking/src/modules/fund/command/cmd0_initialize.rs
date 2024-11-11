use super::{ClaimUnstakedSOLCommand, OperationCommandEntry};
use super::{OperationCommand, OperationCommandContext, SelfExecutable};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(in super::super) struct InitializeCommand {
    // TODO: InitializeCommand
}

impl SelfExecutable for InitializeCommand {
    fn execute(
        &self,
        context: &OperationCommandContext,
        accounts: Vec<&AccountInfo>,
    ) -> Result<Vec<OperationCommandEntry>> {
        // TODO: InitializeCommand.execute
        let staking_stake_command_entry =
            OperationCommand::ClaimUnstakedSOL(ClaimUnstakedSOLCommand {
            // ...
        })
            .build(context)?;
        Ok(vec![staking_stake_command_entry])
    }

    fn compute_required_accounts(&self, context: &OperationCommandContext) -> Result<Vec<Pubkey>> {
        // TODO: InitializeCommand.compute_required_accounts
        Ok(vec![])
    }
}
