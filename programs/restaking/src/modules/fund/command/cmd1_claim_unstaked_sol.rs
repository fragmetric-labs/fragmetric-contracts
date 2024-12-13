use anchor_lang::prelude::*;

use crate::{
    errors,
    modules::{
        pricing::TokenPricingSource,
        staking::{self, SPLStakePoolService},
    },
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
            let mut fund_account = ctx.fund_account.load_mut()?;
            let token = fund_account.get_supported_token(&item.mint)?;

            match &self.state {
                ClaimUnstakedSOLCommandState::Init => {
                    let mut command = self.clone();
                    command.state = ClaimUnstakedSOLCommandState::ReadPoolState;

                    match token.pricing_source.try_deserialize()? {
                        Some(TokenPricingSource::SPLStakePool { address }) => {
                            return Ok(Some(command.with_required_accounts([(address, false)])));
                        }
                        _ => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
                    }
                }
                ClaimUnstakedSOLCommandState::ReadPoolState => {
                    let mut command = self.clone();
                    command.state = ClaimUnstakedSOLCommandState::Claim;

                    let [pool_account_info, ..] = accounts else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };

                    let mut required_accounts = match token.pricing_source.try_deserialize()? {
                        Some(TokenPricingSource::SPLStakePool { address }) => {
                            require_keys_eq!(address, *pool_account_info.key);

                            staking::SPLStakePoolService::find_accounts_to_claim_sol()
                        }
                        _ => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
                    };
                    required_accounts.extend([(fund_account.get_reserve_account_address()?, true)]);
                    required_accounts.extend(
                        item.fund_stake_accounts
                            .iter()
                            .map(|&account| (account, true)),
                    );

                    return Ok(Some(command.with_required_accounts(required_accounts)));
                }
                ClaimUnstakedSOLCommandState::Claim => {
                    let [sysvar_clock_program, sysvar_stake_history_program, stake_program, fund_reserve_account, fund_stake_accounts @ ..] =
                        accounts
                    else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };

                    for (index, fund_stake_account) in fund_stake_accounts
                        .iter()
                        .take(item.fund_stake_accounts.len())
                        .enumerate()
                    {
                        msg!(
                            "fund_stake_account {} key {}, lamports {}",
                            index,
                            fund_stake_account.key,
                            fund_stake_account.lamports()
                        );
                        if fund_stake_account.lamports() > 0 {
                            let received_sol_amount = fund_stake_account.lamports();
                            msg!("Before claim, fund_stake_account lamports {}, fund_reserve_account lamports {}", fund_stake_account.lamports(), fund_reserve_account.lamports());
                            staking::SPLStakePoolService::claim_sol(
                                sysvar_clock_program,
                                sysvar_stake_history_program,
                                stake_program,
                                fund_stake_account,
                                fund_reserve_account,
                                &fund_account.get_reserve_account_seeds(),
                            )?;

                            msg!("After claim, fund_stake_account lamports {}, fund_reserve_account lamports {}", fund_stake_account.lamports(), fund_reserve_account.lamports());

                            fund_account.sol_operation_receivable_amount -= received_sol_amount;
                            fund_account.sol_operation_reserved_amount += received_sol_amount;
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
