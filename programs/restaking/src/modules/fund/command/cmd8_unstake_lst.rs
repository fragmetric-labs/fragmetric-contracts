use anchor_lang::prelude::*;

use crate::modules::pricing::TokenPricingSource;
use crate::modules::staking;
use crate::utils::{AccountExt, PDASeeds};
use crate::{errors, modules::staking::AvailableWithdrawals};

use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct UnstakeLSTCommand {
    #[max_len(10)]
    items: Vec<UnstakeLSTCommandItem>,
    state: UnstakeLSTCommandState,
    #[max_len(5)]
    spl_withdraw_stake_items: Vec<SplWithdrawStakeItem>,
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
            spl_withdraw_stake_items: vec![],
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
    GetAvailableUnstakeAccount,
    Unstake,
    // PrepareRequestUnstake, // 여기서 item에 남은 돈이 있다면, validator 다시 찾아서 RequestUnstake로
    // RequestUnstake(#[max_len(10)] Vec<SplWithdrawStakeItem>),
    RequestUnstake,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct SplWithdrawStakeItem {
    validator_stake_account: Pubkey,
    fund_stake_account: Pubkey, // pda
    #[max_len(4, 32)] // there would be total 3 seeds, max bytes would be 32 bytes per seed
    fund_stake_account_signer_seeds: Vec<Vec<u8>>,
    token_amount: u64,
}

impl SelfExecutable for UnstakeLSTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        // there are remaining tokens to handle
        if let Some(item) = self.items.first() {
            let mut fund_account = ctx.fund_account.load_mut()?;
            let token = fund_account.get_supported_token(&item.mint)?;

            match &self.state {
                UnstakeLSTCommandState::Init if item.token_amount > 0 => {
                    let mut command = self.clone();
                    command.state = UnstakeLSTCommandState::ReadPoolState;

                    match token.pricing_source.try_deserialize()? {
                        TokenPricingSource::SPLStakePool { address }
                        | TokenPricingSource::MarinadeStakePool { address } => {
                            return Ok(Some(command.with_required_accounts([(address, false)])));
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    }
                }
                UnstakeLSTCommandState::ReadPoolState => {
                    let mut command = self.clone();
                    command.state = UnstakeLSTCommandState::GetAvailableUnstakeAccount;

                    let [pool_account_info, ..] = accounts else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };

                    let mut required_accounts = match token.pricing_source.try_deserialize()? {
                        TokenPricingSource::SPLStakePool { address } => {
                            require_keys_eq!(address, *pool_account_info.key);

                            staking::SPLStakePoolService::find_accounts_to_withdraw_sol_or_stake(
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
                        (fund_account.get_reserve_account_address()?, true),
                        (
                            fund_account
                                .find_supported_token_account_address(&item.mint)?,
                            true,
                        ),
                        (ctx.fund_account.key(), true),
                    ]);

                    return Ok(Some(command.with_required_accounts(required_accounts)));
                }
                UnstakeLSTCommandState::GetAvailableUnstakeAccount => {
                    let mut command = self.clone();

                    let [pool_program, pool_account, _pool_token_mint, _pool_token_program, _withdraw_authority, reserve_stake_account, validator_list_account, _manager_fee_account, _sysvar_clock_program, _sysvar_stake_history_program, _stake_program, _fund_reserve_account, _fund_supported_token_account, _fund_account, ..] =
                        accounts
                    else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };

                    let available_withdrawals_from_reserve_or_validator = staking::SPLStakePoolService::get_withdrawal_available_from_reserve_or_validator(pool_program, pool_account, reserve_stake_account, validator_list_account, item.token_amount)?;

                    let mut required_accounts = Vec::new();
                    required_accounts.extend(
                        accounts
                            .iter()
                            .map(|account| (*account.key, account.is_writable)),
                    );

                    if let AvailableWithdrawals::Validators(
                        available_withdrawals_from_reserve_or_validator,
                    ) = available_withdrawals_from_reserve_or_validator
                    {
                        require_neq!(
                            available_withdrawals_from_reserve_or_validator[0].0,
                            Pubkey::default(),
                            errors::ErrorCode::StakingSPLActiveStakeNotAvailableException
                        );
                        command.state = UnstakeLSTCommandState::RequestUnstake;

                        let fund_stake_accounts: Vec<(Pubkey, bool, u8)> = available_withdrawals_from_reserve_or_validator.iter().enumerate().map(|(account_index, _)| {
                            staking::SPLStakePoolService::find_fund_stake_accounts_for_withdraw_stake(&[item.mint.key().as_ref(), pool_account.key.as_ref(), &[account_index as u8]])
                        }).collect();

                        command.spl_withdraw_stake_items =
                            available_withdrawals_from_reserve_or_validator
                                .into_iter()
                                .enumerate()
                                .map(|(index, (validator_stake_account, token_amount))| {
                                    SplWithdrawStakeItem {
                                        validator_stake_account,
                                        fund_stake_account: fund_stake_accounts[index].0,
                                        fund_stake_account_signer_seeds: vec![
                                            item.mint.key().as_ref().to_vec(),
                                            pool_account.key.as_ref().to_vec(),
                                            vec![index as u8],
                                            vec![fund_stake_accounts[index].2],
                                        ],
                                        token_amount,
                                    }
                                })
                                .collect();
                        required_accounts.extend(command.spl_withdraw_stake_items.iter().map(
                            |spl_withdraw_stake_item| {
                                (spl_withdraw_stake_item.validator_stake_account, true)
                            },
                        ));
                        required_accounts.extend(fund_stake_accounts.iter().map(
                            |fund_stake_account| (fund_stake_account.0, fund_stake_account.1),
                        ));
                    } else {
                        command.state = UnstakeLSTCommandState::Unstake;
                    }

                    return Ok(Some(command.with_required_accounts(required_accounts)));
                }
                UnstakeLSTCommandState::Unstake => {
                    // TODO put accounts definition into each token_pricing_source
                    let [pool_program, pool_account, pool_token_mint, pool_token_program, withdraw_authority, reserve_stake_account, _validator_list_account, manager_fee_account, sysvar_clock_program, sysvar_stake_history_program, stake_program, fund_reserve_account, fund_supported_token_account, ..] =
                        accounts
                    else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };

                    let (to_sol_account_amount, returned_sol_amount) = match token.pricing_source.try_deserialize()? {
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
                                ctx.fund_account.as_account_info(),
                                &fund_account.get_seeds(),
                                item.token_amount,
                            )?
                        }
                        TokenPricingSource::MarinadeStakePool { address } => {
                            require_keys_eq!(address, *pool_account.key);

                            todo!() // TODO: support marinade..
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    };

                    fund_account.sol_operation_reserved_amount = fund_account.sol_operation_reserved_amount
                        .checked_add(returned_sol_amount)
                        .ok_or_else(|| {
                            error!(errors::ErrorCode::FundUnexpectedReserveAccountBalanceException)
                        })?;

                    let fund_supported_token_info = fund_account
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
                        fund_account.sol_operation_reserved_amount,
                        to_sol_account_amount
                    );
                }
                UnstakeLSTCommandState::RequestUnstake => {
                    let mut command = self.clone();

                    let [pool_program, pool_account, pool_token_mint, pool_token_program, withdraw_authority, _reserve_stake_account, validator_list_account, manager_fee_account, sysvar_clock_program, _sysvar_stake_history_program, stake_program, fund_reserve_account, fund_supported_token_account, stake_accounts @ ..] =
                        accounts
                    else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };

                    let mut required_accounts = Vec::new();
                    required_accounts.extend(
                        accounts
                            .iter()
                            .map(|account| (*account.key, account.is_writable)),
                    );

                    match token.pricing_source.try_deserialize()? {
                        TokenPricingSource::SPLStakePool { address } => {
                            require_keys_eq!(address, *pool_account.key);

                            let spl_withdraw_stake_item = command.spl_withdraw_stake_items.pop();
                            if let Some(spl_withdraw_stake_item) = spl_withdraw_stake_item {
                                let validator_stake_account = staking::SPLStakePoolService::find_stake_account_info_by_address(stake_accounts, &spl_withdraw_stake_item.validator_stake_account)?;
                                let fund_stake_account = staking::SPLStakePoolService::find_stake_account_info_by_address(stake_accounts, &spl_withdraw_stake_item.fund_stake_account)?;
                                // should create stake account and pass it the cpi call
                                staking::SPLStakePoolService::create_stake_account_if_needed(
                                    fund_reserve_account,
                                    fund_stake_account,
                                    &fund_account.get_reserve_account_seeds(),
                                    &spl_withdraw_stake_item
                                        .fund_stake_account_signer_seeds
                                        .iter()
                                        .map(|seed| seed.as_slice())
                                        .collect::<Vec<&[u8]>>()[..],
                                )?;

                                let returned_sol_amount = staking::SPLStakePoolService::new(
                                    pool_program,
                                    pool_account,
                                    pool_token_mint,
                                    pool_token_program,
                                )?
                                .withdraw_stake(
                                    withdraw_authority,
                                    validator_list_account,
                                    validator_stake_account,
                                    manager_fee_account,
                                    sysvar_clock_program,
                                    stake_program,
                                    fund_supported_token_account,
                                    fund_stake_account,
                                    ctx.fund_account.as_account_info(),
                                    &fund_account.get_seeds(),
                                    spl_withdraw_stake_item.token_amount,
                                )?;
                                msg!("returned_sol_amount {}", returned_sol_amount);

                                if command.spl_withdraw_stake_items.len() > 0 {
                                    command.state = UnstakeLSTCommandState::RequestUnstake;
                                    return Ok(Some(
                                        command.with_required_accounts(required_accounts),
                                    ));
                                }
                            } else {
                                // nothing to do
                            }
                        }
                        TokenPricingSource::MarinadeStakePool { address } => {
                            require_keys_eq!(address, *pool_account.key);

                            todo!() // TODO: support marinade..
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    };
                }
                _ => (),
            }

            // proceed to next token
            if self.items.len() > 1 {
                return Ok(Some(
                    UnstakeLSTCommand::new_init(self.items[1..].to_vec())
                        .without_required_accounts(),
                ));
            }
        }

        // TODO v0.3/operation: next step ... stake sol
        Ok(Some(
            OperationCommand::EnqueueWithdrawalBatch(Default::default())
                .without_required_accounts(),
        ))
    }
}
