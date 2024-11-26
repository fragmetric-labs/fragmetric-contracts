use anchor_lang::prelude::*;

use crate::errors;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::staking;
use crate::utils::PDASeeds;

use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct UnstakeLSTCommand {
    #[max_len(10)]
    items: Vec<UnstakeLSTCommandItem>,
    state: UnstakeLSTCommandState,
}

impl From<UnstakeLSTCommand> for OperationCommand {
    fn from(command: UnstakeLSTCommand) -> Self {
        Self::UnstakeLST(command)
    }
}

impl UnstakeLSTCommand {
    pub(super) fn new_init(items: Vec<UnstakeLSTCommandItem>) -> Self {
        Self {
            items,
            state: UnstakeLSTCommandState::Init,
        }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct UnstakeLSTCommandItem {
    mint: Pubkey,
    token_amount: u64,
}

impl UnstakeLSTCommandItem {
    pub(super) fn new(mint: Pubkey, token_amount: u64) -> Self {
        Self { mint, token_amount }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum UnstakeLSTCommandState {
    Init,
    ReadPoolState,
    Unstake,
}

impl SelfExecutable for UnstakeLSTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        // there are remaining tokens to handle
        if let Some(item) = self.items.first() {
            let token = ctx.fund_account.get_supported_token(&item.mint)?;

            match &self.state {
                UnstakeLSTCommandState::Init if item.token_amount > 0 => {
                    let mut command = self.clone();
                    command.state = UnstakeLSTCommandState::ReadPoolState;

                    match token.pricing_source {
                        TokenPricingSource::SPLStakePool { address }
                        | TokenPricingSource::MarinadeStakePool { address } => {
                            return Ok(Some(command.with_required_accounts([(address, false)])));
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    }
                }
                UnstakeLSTCommandState::ReadPoolState => {
                    let mut command = self.clone();
                    command.state = UnstakeLSTCommandState::Unstake;

                    let [pool_account_info, ..] = accounts else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };

                    let mut required_accounts = match token.pricing_source {
                        TokenPricingSource::SPLStakePool { address } => {
                            require_keys_eq!(address, *pool_account_info.key);

                            staking::SPLStakePoolService::find_accounts_to_withdraw_sol(
                                pool_account_info,
                            )?
                        }
                        TokenPricingSource::MarinadeStakePool { address } => {
                            require_keys_eq!(address, *pool_account_info.key);

                            todo!() // TODO: support marinade..
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    };

                    required_accounts.extend([
                        (ctx.fund_account.get_reserve_account_address()?, true),
                        (
                            ctx.fund_account
                                .find_supported_token_account_address(&item.mint)?,
                            true,
                        ),
                        (ctx.fund_account.key(), true),
                    ]);

                    return Ok(Some(command.with_required_accounts(required_accounts)));
                }
                UnstakeLSTCommandState::Unstake => {
                    let [pool_program, pool_account, pool_token_mint, pool_token_program, withdraw_authority, reserve_stake_account, manager_fee_account, sysvar_clock_program, sysvar_stake_history_program, stake_program, fund_reserve_account, fund_supported_token_account, fund_account, ..] =
                        accounts
                    else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };

                    let (to_sol_account_amount, returned_sol_amount) = match token.pricing_source {
                        TokenPricingSource::SPLStakePool { address } => {
                            require_keys_eq!(address, *pool_account.key);

                            staking::SPLStakePoolService::new(
                                pool_program,
                                pool_account,
                                pool_token_mint,
                                pool_token_program,
                            )?
                            .withdraw_sol(
                                withdraw_authority,
                                reserve_stake_account,
                                manager_fee_account,
                                sysvar_clock_program,
                                sysvar_stake_history_program,
                                stake_program,
                                fund_supported_token_account,
                                fund_reserve_account,
                                fund_account,
                                &ctx.fund_account.get_seeds(),
                                item.token_amount,
                            )?
                        }
                        TokenPricingSource::MarinadeStakePool { address } => {
                            require_keys_eq!(address, *pool_account.key);

                            todo!() // TODO: support marinade..
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    };

                    ctx.fund_account.sol_operation_reserved_amount = ctx
                        .fund_account
                        .sol_operation_reserved_amount
                        .checked_add(returned_sol_amount)
                        .ok_or_else(|| {
                            error!(errors::ErrorCode::FundUnexpectedReserveAccountBalanceException)
                        })?;

                    let fund_supported_token_info = ctx
                        .fund_account
                        .get_supported_token_mut(pool_token_mint.key)?;
                    fund_supported_token_info.set_operation_reserved_amount(
                        fund_supported_token_info
                            .get_operation_reserved_amount()
                            .checked_sub(item.token_amount)
                            .unwrap(),
                    );

                    msg!(
                        "unstaked {} tokens to get {} sol",
                        item.token_amount,
                        returned_sol_amount
                    );

                    require_gte!(returned_sol_amount, item.token_amount);
                    require_eq!(
                        ctx.fund_account.sol_operation_reserved_amount,
                        to_sol_account_amount
                    );
                }
                _ => (),
            }

            // proceed to next token
            if self.items.len() > 1 {
                return Ok(Some(
                    UnstakeLSTCommand::new_init(self.items[1..].to_vec())
                        .with_required_accounts([]),
                ));
            }
        }

        // TODO v0.3/operation: next step ... stake sol
        Ok(Some(
            OperationCommand::EnqueueWithdrawalBatch(Default::default()).with_required_accounts([]),
        ))
    }
}
