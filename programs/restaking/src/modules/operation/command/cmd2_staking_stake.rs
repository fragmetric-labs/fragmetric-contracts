use anchor_lang::prelude::*;
use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct StakingProtocolStakeCommand {
    // add intermediate variables to propagate, refer Section 2. Asset Circulation Breakdown
    pub pool_address: Pubkey,
    pub amount: u64,
}

impl SelfExecutable for StakingProtocolStakeCommand {
    fn execute(
        &self,
        context: &OperationCommandContext,
        accounts: Vec<&AccountInfo>,
    ) -> Result<Vec<OperationCommandEntry>> {
        Ok(vec![]) // at the end of the circulation
    }

    fn compute_required_accounts(
        &self,
        context: &OperationCommandContext,
    ) -> Result<Vec<Pubkey>> {
        Ok(vec![/* ... */])
    }
}