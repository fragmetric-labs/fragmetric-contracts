use super::{ClaimUnstakedSOLCommand, OperationCommandEntry};
use super::{OperationCommand, OperationCommandContext, SelfExecutable};
use crate::constants::{MAINNET_JITOSOL_MINT_ADDRESS, MAINNET_JITOSOL_STAKE_POOL_ADDRESS};
use crate::errors;
use crate::modules::fund;
use crate::modules::fund::command::cmd9_stake_sol::{StakeSOLCommand, StakeSOLCommandState};
use crate::modules::pricing::TokenPricingSource;
use anchor_lang::prelude::*;
use spl_stake_pool::state::StakePool as SPLStakePoolAccount;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct InitializeCommand {}

impl SelfExecutable for InitializeCommand {
    fn execute(
        &self,
        ctx: &mut OperationCommandContext,
        _accounts: &[AccountInfo],
    ) -> Result<Option<OperationCommandEntry>> {
        let mut lst_mints = Vec::new();
        let mut staking_sol_amounts = Vec::new();
        for supported_token in ctx.fund_account.supported_tokens.clone() {
            match supported_token.get_pricing_source() {
                TokenPricingSource::SPLStakePool { .. } => {
                    let mint = supported_token.get_mint();
                    lst_mints.push(mint);

                    // TODO v0.3/operation: stake according to the strategy
                    if mint == MAINNET_JITOSOL_MINT_ADDRESS {
                        staking_sol_amounts.push(ctx.fund_account.sol_operation_reserved_amount);
                    } else {
                        staking_sol_amounts.push(0);
                    }
                }
                TokenPricingSource::MarinadeStakePool { .. } => {
                    lst_mints.push(supported_token.get_mint());
                    staking_sol_amounts.push(0);
                }
                _ => {
                    err!(errors::ErrorCode::OperationCommandAccountComputationException)?;
                }
            }
        }

        // TODO v0.3/operation: follow valid circulation flow...
        Ok(Some(
            OperationCommand::StakeSOL(StakeSOLCommand {
                lst_mints,
                staking_sol_amounts,
                state: StakeSOLCommandState::Init,
            })
            .with_required_accounts(vec![]),
        ))
    }
}
