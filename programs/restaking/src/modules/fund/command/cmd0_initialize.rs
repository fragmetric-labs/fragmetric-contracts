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
        let mut items = Vec::with_capacity(ctx.fund_account.supported_tokens.len());
        for supported_token in ctx.fund_account.supported_tokens.clone() {
            match supported_token.pricing_source {
                TokenPricingSource::SPLStakePool { .. } => {
                    // TODO v0.3/operation: stake according to the strategy
                    let sol_amount = (supported_token.mint == MAINNET_JITOSOL_MINT_ADDRESS)
                        .then_some(ctx.fund_account.sol_operation_reserved_amount)
                        .unwrap_or_default();
                    items.push(StakeSOLCommandItem::new(supported_token.mint, sol_amount));
                }
                TokenPricingSource::MarinadeStakePool { .. } => {
                    // TODO v0.3/staking: support marinade..
                    items.push(StakeSOLCommandItem::new(supported_token.mint, 0));
                }
                _ => {
                    err!(errors::ErrorCode::OperationCommandAccountComputationException)?;
                }
            }
        }

        // TODO v0.3/operation: follow valid circulation flow...
        Ok(Some(
            StakeSOLCommand::new_init(items).with_required_accounts([]),
        ))
    }
}
