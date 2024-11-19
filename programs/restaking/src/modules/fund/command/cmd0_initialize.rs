use super::{ClaimUnstakedSOLCommand, OperationCommandEntry, StakeSOLCommandItem};
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
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        _accounts: &'a [AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        let mut items = Vec::new();
        for supported_token in ctx.fund_account.supported_tokens.clone() {
            match supported_token.get_pricing_source() {
                TokenPricingSource::SPLStakePool { .. } => {
                    let mint = supported_token.get_mint();

                    // TODO v0.3/operation: stake according to the strategy
                    if mint == MAINNET_JITOSOL_MINT_ADDRESS {
                        items.push(StakeSOLCommandItem {
                            mint,
                            sol_amount: ctx.fund_account.sol_operation_reserved_amount,
                        });
                    } else {
                        items.push(StakeSOLCommandItem {
                            mint,
                            sol_amount: 0,
                        });
                    }
                }
                TokenPricingSource::MarinadeStakePool { .. } => {
                    // TODO v0.3/staking: support marinade..
                    items.push(StakeSOLCommandItem {
                        mint: supported_token.get_mint(),
                        sol_amount: 0,
                    });
                }
                _ => {
                    err!(errors::ErrorCode::OperationCommandAccountComputationException)?;
                }
            }
        }

        // TODO v0.3/operation: follow valid circulation flow...
        Ok(Some(
            OperationCommand::StakeSOL(StakeSOLCommand {
                items,
                state: StakeSOLCommandState::Init,
            })
            .with_required_accounts(vec![]),
        ))
    }
}
