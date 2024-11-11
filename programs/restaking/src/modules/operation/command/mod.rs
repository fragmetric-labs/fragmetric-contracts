mod cmd1_initialization;
mod cmd2_staking_stake;

pub use cmd1_initialization::*;
pub use cmd2_staking_stake::*;

use crate::modules::fund;
use anchor_lang::prelude::*;

// propagate common accounts and values to all commands
pub(super) struct OperationCommandContext<'info> {
    fund: &'info mut Account<'info, fund::FundAccount>,
    receipt_token_mint_address: Pubkey,
}

// enum to hold all command variants
#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) enum OperationCommand {
    Initialization(InitializationCommand),
    StakingProtocolStake(StakingProtocolStakeCommand),
}

impl SelfExecutable for OperationCommand {
    fn execute(
        &self,
        context: &OperationCommandContext,
        accounts: Vec<&AccountInfo>,
    ) -> Result<Vec<OperationCommandEntry>> {
        let cmd: &dyn SelfExecutable = match self {
            OperationCommand::Initialization(command) => command,
            OperationCommand::StakingProtocolStake(command) => command,
        };
        cmd.execute(context, accounts)
    }

    fn compute_required_accounts(
        &self,
        context: &OperationCommandContext,
    ) -> Result<Vec<Pubkey>> {
        let cmd: &dyn SelfExecutable = match self {
            OperationCommand::Initialization(command) => command,
            OperationCommand::StakingProtocolStake(command) => command,
        };
        cmd.compute_required_accounts(context)
    }
}

const OPERATION_COMMAND_MAX_ACCOUNT_SIZE: usize = 24;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct OperationCommandEntry {
    pub command: OperationCommand,
    #[max_len(OPERATION_COMMAND_MAX_ACCOUNT_SIZE)]
    pub required_accounts: Vec<Pubkey>,
}

impl OperationCommand {
    pub fn build(&self, context: &OperationCommandContext) -> Result<OperationCommandEntry> {
        Ok(OperationCommandEntry {
            command: self.clone(),
            required_accounts: self.compute_required_accounts(context)?,
        })
    }
}

pub(super) trait SelfExecutable {
    fn execute(
        &self,
        context: &OperationCommandContext,
        accounts: Vec<&AccountInfo>,
    ) -> Result<Vec<OperationCommandEntry>>;

    fn compute_required_accounts(
        &self,
        context: &OperationCommandContext,
    ) -> Result<Vec<Pubkey>>;
}