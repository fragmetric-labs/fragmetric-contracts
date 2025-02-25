use anchor_lang::prelude::*;

use super::{
    OperationCommandContext, OperationCommandEntry, OperationCommandResult, SelfExecutable,
    UnstakeLSTCommand,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct HarvestRewardCommand {}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct HarvestRewardCommandResult {}

impl SelfExecutable for HarvestRewardCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        _ctx: &mut OperationCommandContext<'info, 'a>,
        _accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        // TODO v0.5.0: HarvestRewardCommand.execute
        Ok((
            None,
            Some(UnstakeLSTCommand::default().without_required_accounts()),
        ))
    }
}
