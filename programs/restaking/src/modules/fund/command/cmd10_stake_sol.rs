use super::{
    NormalizeLSTCommand, OperationCommand, OperationCommandContext, OperationCommandEntry,
    OperationCommandResult, SelfExecutable,
};
use crate::constants::MAINNET_JITOSOL_MINT_ADDRESS;
use crate::errors;
use crate::modules::fund::command::OperationCommand::StakeSOL;
use crate::modules::fund::{
    weighted_allocation_strategy, FundService, SupportedToken, WeightedAllocationParticipant,
    WeightedAllocationStrategy, FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
};
use crate::modules::pricing::TokenPricingSource;
use crate::modules::staking;
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct StakeSOLCommand {
    state: StakeSOLCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct StakeSOLCommandItem {
    mint: Pubkey,
    sol_amount: u64,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum StakeSOLCommandState {
    /// Initializes a command with items based on the fund state and strategy.
    #[default]
    New,
    /// Prepares to execute staking for the first item in the list.
    Prepare {
        #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
        items: Vec<StakeSOLCommandItem>,
    },
    /// Executes staking for the first item and transitions to the next command,
    /// either preparing the next item or performing a normalization operation.
    Execute {
        #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
        items: Vec<StakeSOLCommandItem>,
    },
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct StakeSOLCommandResult {
    pub pool_token_mint: Pubkey,
    pub staked_sol_amount: u64,
    pub minted_pool_token_amount: u64,
    pub reserved_pool_token_amount: u64,
}

impl SelfExecutable for StakeSOLCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let mut remaining_items: Option<Vec<StakeSOLCommandItem>> = None;
        let mut result: Option<OperationCommandResult> = None;

        match &self.state {
            StakeSOLCommandState::New => {
                let (pricing_service, sol_net_operation_reserved_amount) = {
                    let mut fund_service =
                        FundService::new(ctx.receipt_token_mint, ctx.fund_account)?;
                    let pricing_service =
                        fund_service.new_pricing_service(accounts.into_iter().cloned())?;
                    let sol_net_operation_reserved_amount = fund_service
                        .get_asset_net_operation_reserved_amount(None, &pricing_service)?;
                    (pricing_service, sol_net_operation_reserved_amount)
                };

                if sol_net_operation_reserved_amount > 0 {
                    let fund_account = ctx.fund_account.load()?;
                    let mut strategy_participants =
                        Vec::<WeightedAllocationParticipant>::with_capacity(
                            FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
                        );
                    let mut strategy_supported_tokens =
                        Vec::<&SupportedToken>::with_capacity(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS);
                    for supported_token in fund_account.get_supported_tokens_iter() {
                        match supported_token.pricing_source.try_deserialize()?.unwrap() {
                            TokenPricingSource::SPLStakePool { .. }
                            | TokenPricingSource::MarinadeStakePool { .. } => {
                                strategy_participants.push(WeightedAllocationParticipant::new(
                                    supported_token.sol_allocation_weight,
                                    pricing_service.get_token_amount_as_sol(
                                        &supported_token.mint,
                                        supported_token.token.operation_reserved_amount,
                                    )?,
                                    supported_token.sol_allocation_capacity_amount,
                                ));
                                strategy_supported_tokens.push(&supported_token)
                            }
                            _ => err!(
                                errors::ErrorCode::FundOperationCommandExecutionFailedException
                            )?,
                        };
                    }
                    let mut strategy = WeightedAllocationStrategy::<
                        FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
                    >::new(strategy_participants);

                    let sol_staking_reserved_amount =
                        u64::try_from(sol_net_operation_reserved_amount)?;
                    let sol_staking_remaining_amount = strategy.put(sol_staking_reserved_amount)?;
                    let _sol_staking_execution_amount =
                        sol_staking_reserved_amount - sol_staking_remaining_amount;

                    let mut items = Vec::<StakeSOLCommandItem>::with_capacity(
                        FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
                    );
                    for (i, supported_token) in strategy_supported_tokens.iter().enumerate() {
                        let sol_amount = strategy.get_participant_last_put_amount_by_index(i)?;
                        if sol_amount > 0 {
                            items.push(StakeSOLCommandItem {
                                mint: supported_token.mint,
                                sol_amount,
                            });
                        }
                    }
                    remaining_items = Some(items);
                }
            }
            StakeSOLCommandState::Prepare { items } => {
                let item = items.first().unwrap();
                let [pool_account, _remaining_accounts @ ..] = accounts else {
                    err!(ErrorCode::AccountNotEnoughKeys)?
                };

                let fund_account = ctx.fund_account.load()?;
                let supported_token = fund_account.get_supported_token(&item.mint)?;
                let mut required_accounts = vec![
                    (fund_account.get_reserve_account_address()?, true),
                    (
                        fund_account.find_supported_token_reserve_account_address(&item.mint)?,
                        true,
                    ),
                ];

                required_accounts.extend(
                    match supported_token.pricing_source.try_deserialize()?.unwrap() {
                        TokenPricingSource::SPLStakePool { address } => {
                            require_keys_eq!(address, pool_account.key());

                            staking::SPLStakePoolService::find_accounts_to_deposit_sol(
                                pool_account,
                            )?
                        }
                        TokenPricingSource::MarinadeStakePool { address } => {
                            require_keys_eq!(address, pool_account.key());

                            staking::MarinadeStakePoolService::find_accounts_to_deposit_sol(
                                pool_account,
                            )?
                        }
                        _ => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
                    },
                );

                return Ok((
                    None,
                    Some(
                        StakeSOLCommand {
                            state: StakeSOLCommandState::Execute {
                                items: items.clone(),
                            },
                        }
                        .with_required_accounts(required_accounts),
                    ),
                ));
            }
            StakeSOLCommandState::Execute { items } => {
                let item = items.first().unwrap();
                remaining_items = Some(items.into_iter().skip(1).copied().collect::<Vec<_>>());

                let token_pricing_source = ctx
                    .fund_account
                    .load()?
                    .get_supported_token(&item.mint)?
                    .pricing_source
                    .try_deserialize()?
                    .unwrap();

                let (
                    pool_token_mint,
                    to_pool_token_account_amount,
                    minted_pool_token_amount,
                    expected_minted_pool_token_amount,
                ) = match token_pricing_source {
                    TokenPricingSource::SPLStakePool { address } => {
                        let [fund_reserve_account, fund_supported_token_reserve_account, pool_program, pool_account, pool_token_mint, pool_token_program, withdraw_authority, reserve_stake_account, manager_fee_account, remaining_accounts @ ..] =
                            accounts
                        else {
                            err!(ErrorCode::AccountNotEnoughKeys)?
                        };

                        require_keys_eq!(address, pool_account.key());

                        // note: assumes zero deposit fee
                        let expected_minted_pool_token_amount =
                            FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                                .new_pricing_service(remaining_accounts.into_iter().cloned())?
                                .get_sol_amount_as_token(pool_token_mint.key, item.sol_amount)?;

                        let fund_account = ctx.fund_account.load()?;
                        let (to_pool_token_account_amount, minted_pool_token_amount) =
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
                                fund_supported_token_reserve_account,
                                &fund_account.get_reserve_account_seeds(),
                                item.sol_amount,
                            )?;

                        (
                            pool_token_mint,
                            to_pool_token_account_amount,
                            minted_pool_token_amount,
                            expected_minted_pool_token_amount,
                        )
                    }
                    TokenPricingSource::MarinadeStakePool { address } => {
                        let [fund_reserve_account, fund_supported_token_reserve_account, pool_program, pool_account, pool_token_mint, pool_token_program, _system_program, liq_pool_sol_leg, liq_pool_token_leg, liq_pool_token_leg_authority, pool_reserve, pool_token_mint_authority, remaining_accounts @ ..] =
                            accounts
                        else {
                            err!(ErrorCode::AccountNotEnoughKeys)?
                        };

                        require_keys_eq!(address, pool_account.key());

                        // note: assumes zero deposit fee
                        let expected_minted_pool_token_amount =
                            FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                                .new_pricing_service(remaining_accounts.into_iter().cloned())?
                                .get_sol_amount_as_token(pool_token_mint.key, item.sol_amount)?;

                        let fund_account = ctx.fund_account.load()?;
                        let (to_pool_token_account_amount, minted_pool_token_amount) =
                            staking::MarinadeStakePoolService::new(
                                pool_program,
                                pool_account,
                                pool_token_mint,
                                pool_token_program,
                            )?
                            .deposit_sol(
                                ctx.system_program,
                                liq_pool_sol_leg,
                                liq_pool_token_leg,
                                liq_pool_token_leg_authority,
                                pool_reserve,
                                pool_token_mint_authority,
                                fund_reserve_account,
                                fund_supported_token_reserve_account,
                                &fund_account.get_reserve_account_seeds(),
                                item.sol_amount,
                            )?;

                        (
                            pool_token_mint,
                            to_pool_token_account_amount,
                            minted_pool_token_amount,
                            expected_minted_pool_token_amount,
                        )
                    }
                    _ => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
                };

                let mut fund_account = ctx.fund_account.load_mut()?;
                fund_account.sol.operation_reserved_amount -= item.sol_amount;

                let supported_token = fund_account.get_supported_token_mut(pool_token_mint.key)?;
                supported_token.token.operation_reserved_amount += minted_pool_token_amount;

                let reserved_pool_token_amount = supported_token.token.get_total_reserved_amount();
                require_gte!(minted_pool_token_amount, expected_minted_pool_token_amount);
                require_gte!(to_pool_token_account_amount, reserved_pool_token_amount);

                result = Some(
                    StakeSOLCommandResult {
                        pool_token_mint: item.mint,
                        staked_sol_amount: item.sol_amount,
                        minted_pool_token_amount,
                        reserved_pool_token_amount,
                    }
                    .into(),
                );
            }
        }

        // transition to next command
        let items = remaining_items.unwrap();
        Ok((
            result,
            Some(if items.len() > 0 {
                let pricing_source = ctx
                    .fund_account
                    .load()?
                    .get_supported_token(&items.first().unwrap().mint)?
                    .pricing_source
                    .try_deserialize()?
                    .ok_or_else(|| {
                        error!(errors::ErrorCode::FundOperationCommandExecutionFailedException)
                    })?;

                StakeSOLCommand {
                    state: StakeSOLCommandState::Prepare { items },
                }
                .with_required_accounts(match pricing_source {
                    TokenPricingSource::SPLStakePool { address }
                    | TokenPricingSource::MarinadeStakePool { address } => {
                        vec![(address, false)]
                    }
                    _ => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
                })
            } else {
                NormalizeLSTCommand::default().without_required_accounts()
            }),
        ))
    }
}
