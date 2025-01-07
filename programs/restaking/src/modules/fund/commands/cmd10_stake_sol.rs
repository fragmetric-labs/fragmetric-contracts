use anchor_lang::prelude::*;

use crate::modules::pricing::TokenPricingSource;
use crate::modules::staking::{
    MarinadeStakePoolService, SPLStakePoolService, SanctumSingleValidatorSPLStakePoolService,
};
use crate::{errors, utils};

use super::{
    FundService, NormalizeSTCommand, OperationCommand, OperationCommandContext,
    OperationCommandEntry, OperationCommandResult, SelfExecutable, SupportedToken,
    WeightedAllocationParticipant, WeightedAllocationStrategy, FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct StakeSOLCommand {
    state: StakeSOLCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct StakeSOLCommandItem {
    token_mint: Pubkey,
    allocated_sol_amount: u64,
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
    pub token_mint: Pubkey,
    pub staked_sol_amount: u64,
    pub deducted_sol_fee_amount: u64,
    pub minted_token_amount: u64,
    pub operation_reserved_token_amount: u64,
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
                let pricing_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                    .new_pricing_service(accounts.into_iter().cloned())?;
                let fund_account = ctx.fund_account.load()?;
                let sol_net_operation_reserved_amount =
                    fund_account.get_asset_net_operation_reserved_amount(None, &pricing_service)?;

                if sol_net_operation_reserved_amount > 0 {
                    let mut strategy_participants =
                        Vec::<WeightedAllocationParticipant>::with_capacity(
                            FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
                        );
                    let mut strategy_supported_tokens =
                        Vec::<&SupportedToken>::with_capacity(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS);
                    for supported_token in fund_account.get_supported_tokens_iter() {
                        match supported_token.pricing_source.try_deserialize()? {
                            Some(TokenPricingSource::SPLStakePool { .. })
                            | Some(TokenPricingSource::MarinadeStakePool { .. })
                            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool {
                                ..
                            }) => {
                                strategy_participants.push(WeightedAllocationParticipant::new(
                                    supported_token.sol_allocation_weight,
                                    fund_account.get_asset_total_amount_as_sol(
                                        Some(supported_token.mint),
                                        &pricing_service,
                                    )?,
                                    supported_token.sol_allocation_capacity_amount,
                                ));
                                strategy_supported_tokens.push(&supported_token)
                            }
                            // otherwise fails
                            Some(TokenPricingSource::JitoRestakingVault { .. })
                            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                            | None => err!(
                                errors::ErrorCode::FundOperationCommandExecutionFailedException
                            )?,
                            #[cfg(all(test, not(feature = "idl-build")))]
                            Some(TokenPricingSource::Mock { .. }) => err!(
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
                                token_mint: supported_token.mint,
                                allocated_sol_amount: sol_amount,
                            });
                        }
                    }
                    if items.len() > 0 {
                        remaining_items = Some(items);
                    }
                }
            }
            StakeSOLCommandState::Prepare { items } => {
                if let Some(item) = items.first() {
                    let fund_account = ctx.fund_account.load()?;
                    let supported_token = fund_account.get_supported_token(&item.token_mint)?;
                    let mut required_accounts = vec![
                        (fund_account.get_reserve_account_address()?, true),
                        (
                            fund_account
                                .find_supported_token_reserve_account_address(&item.token_mint)?,
                            true,
                        ),
                    ];

                    required_accounts.extend(
                        match supported_token.pricing_source.try_deserialize()? {
                            Some(TokenPricingSource::SPLStakePool { address }) => {
                                let [pool_account, _remaining_accounts @ ..] = accounts else {
                                    err!(ErrorCode::AccountNotEnoughKeys)?
                                };
                                require_keys_eq!(address, pool_account.key());

                                <SPLStakePoolService>::find_accounts_to_deposit_sol(pool_account)?
                            }
                            Some(TokenPricingSource::MarinadeStakePool { address }) => {
                                let [pool_account, _remaining_accounts @ ..] = accounts else {
                                    err!(ErrorCode::AccountNotEnoughKeys)?
                                };
                                require_keys_eq!(address, pool_account.key());

                                MarinadeStakePoolService::find_accounts_to_deposit_sol(
                                    pool_account,
                                )?
                            }
                            Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { address }) => {
                                let [pool_account, _remaining_accounts @ ..] = accounts else {
                                    err!(ErrorCode::AccountNotEnoughKeys)?
                                };
                                require_keys_eq!(address, pool_account.key());

                                SanctumSingleValidatorSPLStakePoolService::find_accounts_to_deposit_sol(pool_account)?
                            }
                            // otherwise fails
                            Some(TokenPricingSource::JitoRestakingVault { .. })
                            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                            | None => err!(
                                errors::ErrorCode::FundOperationCommandExecutionFailedException
                            )?,
                            #[cfg(all(test, not(feature = "idl-build")))]
                            Some(TokenPricingSource::Mock { .. }) => err!(
                                errors::ErrorCode::FundOperationCommandExecutionFailedException
                            )?,
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
            }
            StakeSOLCommandState::Execute { items } => {
                if let Some(item) = items.first() {
                    let token_pricing_source = ctx
                        .fund_account
                        .load()?
                        .get_supported_token(&item.token_mint)?
                        .pricing_source
                        .try_deserialize()?;

                    if let Some((
                        pool_token_mint,
                        to_pool_token_account_amount,
                        minted_pool_token_amount,
                        deducted_sol_fee_amount,
                    )) = match token_pricing_source {
                        Some(TokenPricingSource::SPLStakePool { address }) => {
                            let [fund_reserve_account, fund_supported_token_reserve_account, pool_program, pool_account, pool_token_mint, pool_token_program, withdraw_authority, reserve_stake_account, manager_fee_account, _remaining_accounts @ ..] =
                                accounts
                            else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };
                            require_keys_eq!(address, pool_account.key());

                            let fund_account = ctx.fund_account.load()?;
                            let (
                                to_pool_token_account_amount,
                                minted_pool_token_amount,
                                deducted_sol_fee_amount,
                            ) = <SPLStakePoolService>::new(
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
                                item.allocated_sol_amount,
                            )?;

                            Some((
                                pool_token_mint,
                                to_pool_token_account_amount,
                                minted_pool_token_amount,
                                deducted_sol_fee_amount,
                            ))
                        }
                        Some(TokenPricingSource::MarinadeStakePool { address }) => {
                            let [fund_reserve_account, fund_supported_token_reserve_account, pool_program, pool_account, pool_token_mint, pool_token_program, _system_program, liq_pool_sol_leg, liq_pool_token_leg, liq_pool_token_leg_authority, pool_reserve, pool_token_mint_authority, _remaining_accounts @ ..] =
                                accounts
                            else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };
                            require_keys_eq!(address, pool_account.key());

                            if item.allocated_sol_amount
                                < MarinadeStakePoolService::get_min_deposit_sol_amount(
                                    pool_account,
                                )?
                            {
                                None
                            } else {
                                let fund_account = ctx.fund_account.load()?;
                                let (
                                    to_pool_token_account_amount,
                                    minted_pool_token_amount,
                                    deducted_sol_fee_amount,
                                ) = MarinadeStakePoolService::new(
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
                                    item.allocated_sol_amount,
                                )?;

                                Some((
                                    pool_token_mint,
                                    to_pool_token_account_amount,
                                    minted_pool_token_amount,
                                    deducted_sol_fee_amount,
                                ))
                            }
                        }
                        Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool {
                            address,
                        }) => {
                            let [fund_reserve_account, fund_supported_token_reserve_account, pool_program, pool_account, pool_token_mint, pool_token_program, withdraw_authority, reserve_stake_account, manager_fee_account, _remaining_accounts @ ..] =
                                accounts
                            else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };

                            require_keys_eq!(address, pool_account.key());

                            let fund_account = ctx.fund_account.load()?;
                            let (
                                to_pool_token_account_amount,
                                minted_pool_token_amount,
                                deposit_fee,
                            ) = SanctumSingleValidatorSPLStakePoolService::new(
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
                                item.allocated_sol_amount,
                            )?;

                            Some((
                                pool_token_mint,
                                to_pool_token_account_amount,
                                minted_pool_token_amount,
                                deposit_fee,
                            ))
                        }
                        // otherwise fails
                        Some(TokenPricingSource::JitoRestakingVault { .. })
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
                    } {
                        let expected_minted_pool_token_amount =
                            FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                                .new_pricing_service(accounts.into_iter().cloned())?
                                .get_sol_amount_as_token(
                                    pool_token_mint.key,
                                    item.allocated_sol_amount - deducted_sol_fee_amount,
                                )?;

                        let mut fund_account = ctx.fund_account.load_mut()?;
                        fund_account.sol.operation_reserved_amount -= item.allocated_sol_amount;
                        fund_account.sol.operation_receivable_amount += deducted_sol_fee_amount;

                        let supported_token =
                            fund_account.get_supported_token_mut(pool_token_mint.key)?;
                        supported_token.token.operation_reserved_amount += minted_pool_token_amount;

                        require_gte!(minted_pool_token_amount, expected_minted_pool_token_amount);
                        require_gte!(
                            to_pool_token_account_amount,
                            supported_token.token.get_total_reserved_amount()
                        );
                        result = Some(
                            StakeSOLCommandResult {
                                token_mint: item.token_mint,
                                staked_sol_amount: item.allocated_sol_amount,
                                deducted_sol_fee_amount,
                                minted_token_amount: minted_pool_token_amount,
                                operation_reserved_token_amount: supported_token
                                    .token
                                    .operation_reserved_amount,
                            }
                            .into(),
                        );
                    }

                    remaining_items = Some(items.into_iter().skip(1).copied().collect::<Vec<_>>());
                }
            }
        }

        // transition to next command
        Ok((
            result,
            Some(match remaining_items {
                Some(remaining_items) if remaining_items.len() > 0 => {
                    let pricing_source = ctx
                        .fund_account
                        .load()?
                        .get_supported_token(&remaining_items.first().unwrap().token_mint)?
                        .pricing_source
                        .try_deserialize()?;

                    StakeSOLCommand {
                        state: StakeSOLCommandState::Prepare {
                            items: remaining_items,
                        },
                    }
                    .with_required_accounts(match pricing_source {
                        Some(TokenPricingSource::SPLStakePool { address })
                        | Some(TokenPricingSource::MarinadeStakePool { address })
                        | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool {
                            address,
                        }) => {
                            vec![(address, false)]
                        }
                        // otherwise fails
                        Some(TokenPricingSource::JitoRestakingVault { .. })
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
                    })
                }
                _ => NormalizeSTCommand::default().without_required_accounts(),
            }),
        ))
    }
}
