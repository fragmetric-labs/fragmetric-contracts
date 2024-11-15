use super::{ClaimUnstakedSOLCommand, OperationCommandEntry};
use super::{OperationCommand, OperationCommandContext, SelfExecutable};
use crate::errors;
use crate::modules::fund;
use crate::modules::fund::command::cmd9_stake_sol::{StakeSOLCommand, StakeSOLCommandState};
use crate::modules::pricing::TokenPricingSource;
use anchor_lang::prelude::*;
use spl_stake_pool::state::StakePool as SPLStakePoolAccount;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(in super::super) struct InitializeCommand {}

impl SelfExecutable for InitializeCommand {
    fn execute(
        &self,
        ctx: &mut OperationCommandContext,
        _accounts: &[AccountInfo],
    ) -> Result<Vec<OperationCommandEntry>> {
        let mut supported_lst_mints = Vec::new();
        for supported_token in ctx.fund_account.supported_tokens.clone() {
            match supported_token.get_pricing_source() {
                TokenPricingSource::SPLStakePool { address } => {
                    supported_lst_mints.push(supported_token.get_mint());
                }
                TokenPricingSource::MarinadeStakePool { address } => {
                    supported_lst_mints.push(supported_token.get_mint());
                }
                _ => {
                    err!(errors::ErrorCode::OperationCommandAccountComputationException)?;
                }
            }
        }

        Ok(vec![OperationCommand::StakeSOL(StakeSOLCommand {
            remaining_lst_mints: supported_lst_mints,
            state: StakeSOLCommandState::Init,
        })
        .with_required_accounts(vec![])])
    }
}
