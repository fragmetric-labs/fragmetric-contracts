use anchor_lang::prelude::*;

use crate::constants::MAINNET_JITOSOL_MINT_ADDRESS;
use crate::errors;
use crate::modules::pricing::TokenPricingSource;

use super::cmd9_stake_sol::{StakeSOLCommand, StakeSOLCommandItem, StakeSOLCommandState};
use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct InitializeCommand {}

impl From<InitializeCommand> for OperationCommand {
    fn from(command: InitializeCommand) -> Self {
        Self::Initialize(command)
    }
}

impl SelfExecutable for InitializeCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        _accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        // TODO v0.3/operation: proceed to claim_unstaked_sol command
        Ok(None)
    }
}
