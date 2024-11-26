use anchor_lang::prelude::*;

use crate::errors;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::staking;

use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct StakeSOLCommand {
    #[max_len(10)]
    items: Vec<StakeSOLCommandItem>,
    state: StakeSOLCommandState,
}

impl From<StakeSOLCommand> for OperationCommand {
    fn from(command: StakeSOLCommand) -> Self {
        Self::StakeSOL(command)
    }
}

impl StakeSOLCommand {
    pub(super) fn new_init(items: Vec<StakeSOLCommandItem>) -> Self {
        Self {
            items,
            state: StakeSOLCommandState::Init,
        }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct StakeSOLCommandItem {
    mint: Pubkey,
    sol_amount: u64,
}

impl StakeSOLCommandItem {
    pub(super) fn new(mint: Pubkey, sol_amount: u64) -> Self {
        Self { mint, sol_amount }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum StakeSOLCommandState {
    Init,
    ReadPoolState,
    Stake,
}

impl SelfExecutable for StakeSOLCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        // there are remaining tokens to handle
        if let Some(item) = self.items.first() {
            let token = ctx.fund_account.get_supported_token(&item.mint)?;

            match &self.state {
                StakeSOLCommandState::Init if item.sol_amount > 0 => {
                    let mut command = self.clone();
                    command.state = StakeSOLCommandState::ReadPoolState;

                    match token.pricing_source {
                        TokenPricingSource::SPLStakePool { address }
                        | TokenPricingSource::MarinadeStakePool { address } => {
                            return Ok(Some(command.with_required_accounts([(address, false)])));
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    }
                }
                StakeSOLCommandState::ReadPoolState => {
                    let mut command = self.clone();
                    command.state = StakeSOLCommandState::Stake;

                    let [pool_account_info, _remaining_accounts @ ..] = accounts else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };

                    let mut required_accounts = match token.pricing_source {
                        TokenPricingSource::SPLStakePool { address } => {
                            require_keys_eq!(address, *pool_account_info.key);

                            staking::SPLStakePoolService::find_accounts_to_deposit_sol(
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
                    ]);

                    return Ok(Some(command.with_required_accounts(required_accounts)));
                }
                StakeSOLCommandState::Stake => {
                    let [pool_program, pool_account, pool_token_mint, pool_token_program, withdraw_authority, reserve_stake_account, manager_fee_account, fund_reserve_account, fund_supported_token_account, _remaining_accounts @ ..] =
                        accounts
                    else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };

                    let (to_pool_token_account_amount, minted_supported_token_amount) =
                        match token.pricing_source {
                            TokenPricingSource::SPLStakePool { address } => {
                                require_keys_eq!(address, *pool_account.key);

                                staking::SPLStakePoolService::new(
                                    pool_program,
                                    pool_account,
                                    pool_token_mint,
                                    pool_token_program,
                                )?
                                .deposit_sol(
                                    withdraw_authority,
                                    reserve_stake_account,
                                    manager_fee_account,
                                    fund_reserve_account,
                                    fund_supported_token_account,
                                    &ctx.fund_account.get_reserve_account_seeds(),
                                    item.sol_amount,
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
                        .checked_sub(item.sol_amount)
                        .ok_or_else(|| {
                            error!(errors::ErrorCode::FundUnexpectedReserveAccountBalanceException)
                        })?;

                    let fund_supported_token_info = ctx
                        .fund_account
                        .get_supported_token_mut(pool_token_mint.key)?;
                    fund_supported_token_info.set_operation_reserved_amount(
                        fund_supported_token_info
                            .get_operation_reserved_amount()
                            .checked_add(minted_supported_token_amount)
                            .unwrap(),
                    );

                    msg!(
                        "staked {} sol to mint {} tokens",
                        item.sol_amount,
                        minted_supported_token_amount
                    );

                    require_gte!(minted_supported_token_amount, item.sol_amount.div_ceil(2));
                    require_eq!(
                        fund_supported_token_info.get_operation_reserved_amount(),
                        to_pool_token_account_amount
                    );
                }
                _ => (),
            }

            // proceed to next token
            if self.items.len() > 1 {
                return Ok(Some(
                    StakeSOLCommand::new_init(self.items[1..].to_vec()).with_required_accounts([]),
                ));
            }
        }

        // TODO v0.3/operation: next step after stake sol
        Ok(None)
    }
}
