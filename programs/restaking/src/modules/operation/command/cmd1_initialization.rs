use anchor_lang::prelude::*;
use super::{OperationCommandEntry, StakingProtocolStakeCommand};
use super::{OperationCommand, OperationCommandContext, SelfExecutable};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(in crate::modules::operation) struct InitializationCommand {
}

impl SelfExecutable for InitializationCommand {
    fn execute(
        &self,
        context: &OperationCommandContext,
        accounts: Vec<&AccountInfo>,
    ) -> Result<Vec<OperationCommandEntry>> {
        let staking_stake_command_entry = OperationCommand::StakingProtocolStake(StakingProtocolStakeCommand {
            pool_address: Default::default(),
            amount: 0,
        }).build(context)?;
        Ok(vec![staking_stake_command_entry])
    }

    fn compute_required_accounts(
        &self,
        context: &OperationCommandContext,
    ) -> Result<Vec<Pubkey>> {
        Ok(vec![/* ... */])
    }
}
