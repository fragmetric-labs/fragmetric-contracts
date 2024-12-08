use anchor_lang::prelude::*;

use crate::{
    errors,
    modules::{pricing::TokenPricingSource, staking},
};

use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ClaimUnstakedSOLCommand {
    #[max_len(10)]
    items: Vec<ClaimUnstakedSOLCommandItem>,
    state: ClaimUnstakedSOLCommandState,
}

impl From<ClaimUnstakedSOLCommand> for OperationCommand {
    fn from(command: ClaimUnstakedSOLCommand) -> Self {
        Self::ClaimUnstakedSOL(command)
    }
}

impl ClaimUnstakedSOLCommand {
    pub(super) fn new_init(items: Vec<ClaimUnstakedSOLCommandItem>) -> Self {
        Self {
            items,
            state: ClaimUnstakedSOLCommandState::Init,
        }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ClaimUnstakedSOLCommandItem {
    mint: Pubkey,
    #[max_len(5)]
    fund_stake_accounts: Vec<Pubkey>,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum ClaimUnstakedSOLCommandState {
    Init,
    ReadPoolState,
    Claim,
}

impl SelfExecutable for ClaimUnstakedSOLCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        if let Some(item) = self.items.first() {
            let fund_account = ctx.fund_account.load()?;
            let token = fund_account.get_supported_token(&item.mint)?;

            match &self.state {
                ClaimUnstakedSOLCommandState::Init => {
                    let mut command = self.clone();
                    command.state = ClaimUnstakedSOLCommandState::ReadPoolState;

                    match token.pricing_source.into() {
                        Some(TokenPricingSource::SPLStakePool { address }) => {
                            return Ok(Some(command.with_required_accounts([(address, false)])));
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    }
                }
                ClaimUnstakedSOLCommandState::ReadPoolState => {
                    let mut command = self.clone();
                    command.state = ClaimUnstakedSOLCommandState::Claim;

                    let [pool_account_info, ..] = accounts else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };

                    let mut required_accounts = match token.pricing_source.into() {
                        Some(TokenPricingSource::SPLStakePool { address }) => {
                            require_keys_eq!(address, *pool_account_info.key);

                            staking::SPLStakePoolService::find_accounts_to_withdraw_sol_or_stake(
                                pool_account_info,
                            )?
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    };
                    required_accounts.extend(
                        item.fund_stake_accounts
                            .iter()
                            .map(|&account| (account, true)),
                    );

                    return Ok(Some(command.with_required_accounts(required_accounts)));
                }
                ClaimUnstakedSOLCommandState::Claim => {
                    let mut command = self.clone();

                    let [pool_program, pool_account, pool_token_mint, pool_token_program, _withdraw_authority, reserve_stake_account, validator_list_account, _manager_fee_account, _sysvar_clock_program, _sysvar_stake_history_program, _stake_program, fund_stake_accounts @ ..] =
                        accounts
                    else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };

                    for stake_account in fund_stake_accounts {
                        msg!(
                            "fund_stake_account key {}, lamports {}",
                            stake_account.key,
                            stake_account.lamports()
                        );
                        if stake_account.lamports() > 0 {
                            staking::SPLStakePoolService::new(
                                pool_program,
                                pool_account,
                                pool_token_mint,
                                pool_token_program,
                            )?
                            .claim_sol(stake_account)?;
                        }
                    }
                }
                _ => (),
            }

            // proceed to next token
            if self.items.len() > 1 {
                return Ok(Some(
                    ClaimUnstakedSOLCommand::new_init(self.items[1..].to_vec())
                        .without_required_accounts(),
                ));
            }
        }

        Ok(None)
    }
}
