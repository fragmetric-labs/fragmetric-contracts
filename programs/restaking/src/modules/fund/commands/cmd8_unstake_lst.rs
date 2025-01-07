use anchor_lang::prelude::*;
use anchor_spl::token::Token;

use crate::errors;
use crate::modules::fund::FundService;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::staking::{
    AvailableWithdrawals, SPLStakePool, SPLStakePoolInterface, SPLStakePoolService,
    SanctumSingleValidatorSPLStakePool, SanctumSingleValidatorSPLStakePoolService,
};
use crate::utils::{AsAccountInfo, PDASeeds};

use super::{
    OperationCommand, OperationCommandContext, OperationCommandEntry, OperationCommandResult,
    SelfExecutable,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct UnstakeLSTCommand {
    #[max_len(10)]
    items: Vec<UnstakeLSTCommandItem>,
    state: UnstakeLSTCommandState,
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
    GetAvailableUnstakeAccount,
    Unstake,
    RequestUnstake(#[max_len(5)] Vec<WithdrawStakeItem>),
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct WithdrawStakeItem {
    validator_stake_account: Pubkey,
    fund_stake_account: Pubkey, // pda
    #[max_len(4, 32)] // there would be total 3 seeds, max bytes would be 32 bytes per seed
    fund_stake_account_signer_seeds: Vec<Vec<u8>>,
    token_amount: u64,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct UnstakeLSTCommandResult {}

impl SelfExecutable for UnstakeLSTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        // there are remaining tokens to handle
        if let Some(item) = self.items.first() {
            match &self.state {
                UnstakeLSTCommandState::Init if item.token_amount > 0 => {
                    return self.execute_init(ctx, item);
                }
                UnstakeLSTCommandState::ReadPoolState => {
                    return self.execute_read_pool_state(ctx, accounts, item);
                }
                UnstakeLSTCommandState::GetAvailableUnstakeAccount => {
                    return self.execute_get_available_unstake_account(ctx, accounts, item);
                }
                UnstakeLSTCommandState::Unstake => {
                    self.execute_unstake(ctx, accounts, item)?;
                }
                UnstakeLSTCommandState::RequestUnstake(withdraw_stake_items) => {
                    self.execute_request_unstake(ctx, accounts, item, withdraw_stake_items)?;

                    if withdraw_stake_items.len() > 1 {
                        let mut command = self.clone();
                        command.state = UnstakeLSTCommandState::RequestUnstake(
                            withdraw_stake_items[1..].to_vec(),
                        );

                        return Ok((
                            None,
                            Some(
                                command.with_required_accounts(
                                    accounts
                                        .iter()
                                        .map(|account| (*account.key, account.is_writable)),
                                ),
                            ),
                        ));
                    }
                }
                _ => (), // state = INIT but amount == 0
            }

            // proceed to next token
            if self.items.len() > 1 {
                return Ok((
                    None,
                    Some(
                        UnstakeLSTCommand::new_init(self.items[1..].to_vec())
                            .without_required_accounts(),
                    ),
                ));
            }
        }

        // TODO v0.3/operation: next step ... stake sol
        Ok((
            None,
            Some(
                OperationCommand::EnqueueWithdrawalBatch(Default::default())
                    .without_required_accounts(),
            ),
        ))
    }
}

impl UnstakeLSTCommand {
    fn execute_init<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        item: &UnstakeLSTCommandItem,
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let mut command = self.clone();
        command.state = UnstakeLSTCommandState::ReadPoolState;

        match ctx
            .fund_account
            .load()?
            .get_supported_token(&item.mint)?
            .pricing_source
            .try_deserialize()?
        {
            Some(TokenPricingSource::SPLStakePool { address })
            | Some(TokenPricingSource::MarinadeStakePool { address })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { address }) => {
                return Ok((
                    None,
                    Some(command.with_required_accounts([(address, false)])),
                ));
            }
            // otherwise fails
            Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | None => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        }
    }

    fn execute_read_pool_state<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
        item: &UnstakeLSTCommandItem,
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let [pool_account_info, ..] = accounts else {
            err!(ErrorCode::AccountNotEnoughKeys)?
        };

        let mut command = self.clone();
        let required_accounts = match ctx
            .fund_account
            .load()?
            .get_supported_token(&item.mint)?
            .pricing_source
            .try_deserialize()?
        {
            Some(TokenPricingSource::MarinadeStakePool { address }) => {
                require_keys_eq!(address, *pool_account_info.key);

                todo!() // TODO: support marinade..
            }
            Some(TokenPricingSource::SPLStakePool { address }) => {
                require_keys_eq!(address, *pool_account_info.key);

                command.state = UnstakeLSTCommandState::GetAvailableUnstakeAccount;
                <SPLStakePoolService>::find_accounts_to_get_available_unstake_account(
                    pool_account_info,
                )?
            }
            Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { address }) => {
                require_keys_eq!(address, *pool_account_info.key);

                command.state = UnstakeLSTCommandState::GetAvailableUnstakeAccount;
                SanctumSingleValidatorSPLStakePoolService::find_accounts_to_get_available_unstake_account(pool_account_info)?
            }
            // otherwise fails
            Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | None => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        };

        return Ok((
            None,
            Some(command.with_required_accounts(required_accounts)),
        ));
    }

    fn execute_get_available_unstake_account<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
        item: &UnstakeLSTCommandItem,
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let supported_token_pricing_source = ctx
            .fund_account
            .load()?
            .get_supported_token(&item.mint)?
            .pricing_source
            .try_deserialize()?;
        match supported_token_pricing_source {
            Some(TokenPricingSource::SPLStakePool { address }) => self
                .get_available_unstake_account_from_spl_stake_pool::<SPLStakePool>(
                    ctx, accounts, item, address
                ),
            Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { address }) => self
                .get_available_unstake_account_from_spl_stake_pool::<SanctumSingleValidatorSPLStakePool>(ctx, accounts, item, address),
            // otherwise fails
            Some(TokenPricingSource::MarinadeStakePool { .. })
            | Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | None => {
                err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
            }
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        }
    }

    fn get_available_unstake_account_from_spl_stake_pool<'a, 'info, T>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
        item: &UnstakeLSTCommandItem,
        pool_account_address: Pubkey,
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )>
    where
        'info: 'a,
        T: SPLStakePoolInterface,
    {
        let mut command = self.clone();

        // stake_program is not used directly, but it needs when running transaction
        let [pool_program, pool_account, pool_token_mint, pool_token_program, reserve_stake_account, validator_list_account, stake_program, ..] =
            accounts
        else {
            err!(ErrorCode::AccountNotEnoughKeys)?
        };

        require_keys_eq!(pool_account_address, pool_account.key());

        let fund_account = ctx.fund_account.load()?;
        let mut required_accounts = vec![
            (pool_program.key(), pool_program.is_writable),
            (pool_account.key(), pool_account.is_writable),
            (pool_token_mint.key(), pool_token_mint.is_writable),
            (pool_token_program.key(), pool_token_program.is_writable),
            (
                reserve_stake_account.key(),
                reserve_stake_account.is_writable,
            ),
            (
                validator_list_account.key(),
                validator_list_account.is_writable,
            ),
            (stake_program.key(), stake_program.is_writable),
            (fund_account.get_reserve_account_address()?, true),
            (
                fund_account.find_supported_token_reserve_account_address(&item.mint)?,
                true,
            ),
        ];

        let available_withdrawals_from_reserve_or_validator =
            SPLStakePoolService::<T>::get_withdrawal_available_from_reserve_or_validator(
                pool_program,
                pool_account,
                reserve_stake_account,
                validator_list_account,
                item.token_amount,
            )?;

        let required_withdraw_sol_or_stake_accounts =
            SPLStakePoolService::<T>::find_accounts_to_withdraw_sol_or_stake(pool_account)?;
        required_accounts.extend(required_withdraw_sol_or_stake_accounts);

        if let AvailableWithdrawals::Validators(available_withdrawals_from_reserve_or_validator) =
            available_withdrawals_from_reserve_or_validator
        {
            require_neq!(
                available_withdrawals_from_reserve_or_validator[0].0,
                Pubkey::default(),
                errors::ErrorCode::StakingSPLActiveStakeNotAvailableException
            );

            let fund_stake_accounts = available_withdrawals_from_reserve_or_validator
                .iter()
                .enumerate()
                .map(|(account_index, _)| {
                    SPLStakePoolService::<T>::find_fund_stake_accounts_for_withdraw_stake(&[
                        ctx.fund_account.key().as_ref(),
                        pool_account.key.as_ref(),
                        &[account_index as u8],
                    ])
                })
                .collect::<Vec<_>>();

            let withdraw_stake_items = available_withdrawals_from_reserve_or_validator
                .into_iter()
                .enumerate()
                .map(
                    |(index, (validator_stake_account, token_amount))| WithdrawStakeItem {
                        validator_stake_account,
                        fund_stake_account: fund_stake_accounts[index].0,
                        fund_stake_account_signer_seeds: vec![
                            ctx.fund_account.key().as_ref().to_vec(),
                            pool_account.key.as_ref().to_vec(),
                            vec![index as u8],
                            vec![fund_stake_accounts[index].2],
                        ],
                        token_amount,
                    },
                )
                .collect::<Vec<_>>();
            required_accounts.extend(
                withdraw_stake_items
                    .iter()
                    .map(|item| (item.validator_stake_account, true)),
            );
            required_accounts.extend(
                fund_stake_accounts
                    .iter()
                    .map(|fund_stake_account| (fund_stake_account.0, fund_stake_account.1)),
            );
            command.state = UnstakeLSTCommandState::RequestUnstake(withdraw_stake_items);
        } else {
            command.state = UnstakeLSTCommandState::Unstake;
        }

        return Ok((
            None,
            Some(command.with_required_accounts(required_accounts)),
        ));
    }

    fn execute_unstake<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
        item: &UnstakeLSTCommandItem,
    ) -> Result<()> {
        let supported_token_pricing_source = ctx
            .fund_account
            .load()?
            .get_supported_token(&item.mint)?
            .pricing_source
            .try_deserialize()?;
        match supported_token_pricing_source {
            Some(TokenPricingSource::SPLStakePool { address }) => {
                self.unstake_from_spl_stake_pool::<SPLStakePool>(ctx, accounts, item, address)
            }
            Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { address }) => {
                self.unstake_from_spl_stake_pool::<SanctumSingleValidatorSPLStakePool>(
                    ctx, accounts, item, address,
                )
            }
            // otherwise fails
            Some(TokenPricingSource::MarinadeStakePool { .. })
            | Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | None => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        }
    }

    fn unstake_from_spl_stake_pool<'a, 'info, T>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
        item: &UnstakeLSTCommandItem,
        pool_account_address: Pubkey,
    ) -> Result<()>
    where
        'info: 'a,
        T: SPLStakePoolInterface,
    {
        let [pool_program, pool_account, pool_token_mint, pool_token_program, reserve_stake_account, _validator_list_account, stake_program, fund_reserve_account, fund_supported_token_reserve_account, withdraw_authority, manager_fee_account, sysvar_clock_program, sysvar_stake_history_program, ..] =
            accounts
        else {
            err!(ErrorCode::AccountNotEnoughKeys)?
        };

        require_keys_eq!(pool_account_address, *pool_account.key);

        let (to_sol_account_amount, returned_sol_amount) = {
            let fund_account = ctx.fund_account.load()?;
            SPLStakePoolService::<T>::new(
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
                fund_supported_token_reserve_account,
                fund_reserve_account,
                ctx.fund_account.as_account_info(),
                &fund_account.get_seeds(),
                item.token_amount,
            )?
        };

        let mut fund_account = ctx.fund_account.load_mut()?;
        fund_account.sol.operation_reserved_amount =
            fund_account.sol.operation_reserved_amount + returned_sol_amount;

        let supported_token = fund_account.get_supported_token_mut(pool_token_mint.key)?;
        supported_token.token.operation_reserved_amount =
            supported_token.token.operation_reserved_amount - item.token_amount;

        msg!(
            "unstaked {} tokens to get {} sol",
            item.token_amount,
            returned_sol_amount
        );

        require_gte!(returned_sol_amount, item.token_amount);
        require_eq!(
            fund_account.sol.operation_reserved_amount,
            to_sol_account_amount
        );

        Ok(())
    }

    fn execute_request_unstake<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
        item: &UnstakeLSTCommandItem,
        withdraw_stake_items: &Vec<WithdrawStakeItem>,
    ) -> Result<()> {
        let supported_token_pricing_source = ctx
            .fund_account
            .load()?
            .get_supported_token(&item.mint)?
            .pricing_source
            .try_deserialize()?;
        match supported_token_pricing_source {
            Some(TokenPricingSource::SPLStakePool { address }) => self
                .request_unstake_from_spl_stake_pool::<SPLStakePool>(
                    ctx,
                    accounts,
                    withdraw_stake_items,
                    address,
                ),
            Some(TokenPricingSource::MarinadeStakePool { .. }) => {
                // require_keys_eq!(address, pool_account.key());

                todo!() // TODO: support marinade..
            }
            Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { address }) => {
                self.request_unstake_from_spl_stake_pool::<SanctumSingleValidatorSPLStakePool>(
                    ctx,
                    accounts,
                    withdraw_stake_items,
                    address,
                )
            }
            // otherwise fails
            Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | None => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        }
    }

    fn request_unstake_from_spl_stake_pool<'a, 'info, T>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
        withdraw_stake_items: &Vec<WithdrawStakeItem>,
        pool_account_address: Pubkey,
    ) -> Result<()>
    where
        'info: 'a,
        T: SPLStakePoolInterface,
    {
        let [pool_program, pool_account, pool_token_mint, pool_token_program, _reserve_stake_account, validator_list_account, stake_program, fund_reserve_account, fund_supported_token_reserve_account, withdraw_authority, manager_fee_account, sysvar_clock_program, _sysvar_stake_history_program, stake_accounts_with_remainings @ ..] =
            accounts
        else {
            err!(ErrorCode::AccountNotEnoughKeys)?
        };

        require_keys_eq!(pool_account_address, *pool_account.key);

        if let Some(withdraw_stake_item) = withdraw_stake_items.first() {
            let fund_account = ctx.fund_account.load()?;
            let validator_stake_account =
                SPLStakePoolService::<T>::find_stake_account_info_by_address(
                    stake_accounts_with_remainings,
                    &withdraw_stake_item.validator_stake_account,
                )?;
            let fund_stake_account = SPLStakePoolService::<T>::find_stake_account_info_by_address(
                stake_accounts_with_remainings,
                &withdraw_stake_item.fund_stake_account,
            )?;

            // should create stake account and pass it the cpi call
            SPLStakePoolService::<T>::create_stake_account_if_needed(
                fund_reserve_account,
                fund_stake_account,
                &fund_account.get_reserve_account_seeds(),
                &withdraw_stake_item
                    .fund_stake_account_signer_seeds
                    .iter()
                    .map(|seed| seed.as_slice())
                    .collect::<Vec<&[u8]>>()[..],
                ctx.system_program,
            )?;

            let returned_sol_amount = SPLStakePoolService::<T>::new(
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
                fund_supported_token_reserve_account,
                fund_stake_account,
                ctx.fund_account.as_account_info(),
                &fund_account.get_seeds(),
                fund_reserve_account,
                withdraw_stake_item.token_amount,
            )?;
            msg!("returned_sol_amount {}", returned_sol_amount);

            // deactivate fund_stake_account
            SPLStakePoolService::<T>::deactivate_stake_account(
                sysvar_clock_program,
                fund_stake_account,
                fund_reserve_account,
                &fund_account.get_reserve_account_seeds(),
            )?;
            drop(fund_account);

            // returned_sol_amount + pool_token_fee as sol
            let receivable_sol_amount = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                .new_pricing_service(
                    [&stake_accounts_with_remainings[..], &[pool_account]] // TODO stake_accounts includes remaining_accounts, should be redefined..
                        .concat()
                        .into_iter(),
                )?
                .get_token_amount_as_sol(pool_token_mint.key, withdraw_stake_item.token_amount)?;

            let mut fund_account = ctx.fund_account.load_mut()?;
            let supported_token = fund_account.get_supported_token_mut(pool_token_mint.key)?;
            supported_token.token.operation_reserved_amount -= withdraw_stake_item.token_amount;
            fund_account.sol.operation_receivable_amount += receivable_sol_amount;
        }

        Ok(())
    }
}
