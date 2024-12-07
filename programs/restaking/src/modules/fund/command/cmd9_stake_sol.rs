use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};
use crate::constants::MAINNET_JITOSOL_MINT_ADDRESS;
use crate::errors;
use crate::modules::fund::command::OperationCommand::StakeSOL;
use crate::modules::fund::{
    weighted_allocation_strategy, FundService, WeightedAllocationParticipant,
    WeightedAllocationStrategy,
};
use crate::modules::pricing::TokenPricingSource;
use crate::modules::staking;
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct StakeSOLCommand {
    #[max_len(10)]
    items: Vec<StakeSOLCommandItem>,
    state: StakeSOLCommandState,
}

impl From<StakeSOLCommand> for OperationCommand {
    fn from(command: StakeSOLCommand) -> Self {
        StakeSOL(command)
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct StakeSOLCommandItem {
    mint: Pubkey,
    sol_amount: u64,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum StakeSOLCommandState {
    /// populate a command with items based on the fund state and operational strategy.
    #[default]
    New,
    /// initialize the command state to process a stacked item.
    Init,
    /// read the stake pool state to determine the required accounts for processing the given item.
    ReadPoolState,
    /// perform the staking operation for the given item.
    Stake,
}

impl Default for StakeSOLCommand {
    fn default() -> StakeSOLCommand {
        Self {
            items: vec![],
            state: StakeSOLCommandState::default(),
        }
    }
}

impl SelfExecutable for StakeSOLCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        match &self.state {
            StakeSOLCommandState::New => {
                let (pricing_service, sol_staking_reserved_amount) = {
                    let mut fund_service =
                        FundService::new(ctx.receipt_token_mint, ctx.fund_account)?;
                    let pricing_service =
                        fund_service.new_pricing_service(accounts.into_iter().cloned())?;
                    let sol_staking_reserved_amount =
                        fund_service.get_sol_staking_reserved_amount(&pricing_service)?;
                    (pricing_service, sol_staking_reserved_amount)
                };

                if sol_staking_reserved_amount > 0 {
                    let fund_account = ctx.fund_account.load()?;
                    let mut participants = fund_account
                        .supported_tokens
                        .iter()
                        .map(|supported_token| {
                            Ok::<WeightedAllocationParticipant, Error>(
                                WeightedAllocationParticipant::new(
                                    supported_token.sol_allocation_weight,
                                    pricing_service.get_token_amount_as_sol(
                                        &supported_token.mint,
                                        supported_token.operation_reserved_amount,
                                    )?,
                                    pricing_service.get_token_amount_as_sol(
                                        &supported_token.mint,
                                        supported_token.accumulated_deposit_capacity_amount,
                                    )?,
                                ),
                            )
                        })
                        .collect::<Result<Vec<_>>>()?;
                    let sol_staking_reserved_amount_positive =
                        u64::try_from(sol_staking_reserved_amount)?;
                    let sol_staking_remaining_amount = WeightedAllocationStrategy::put(
                        &mut *participants,
                        sol_staking_reserved_amount_positive,
                    );
                    let _sol_staking_execution_amount =
                        sol_staking_reserved_amount_positive - sol_staking_remaining_amount;

                    let items = fund_account
                        .supported_tokens
                        .iter()
                        .enumerate()
                        .map(|(i, supported_token)| {
                            Ok(StakeSOLCommandItem {
                                mint: supported_token.mint,
                                sol_amount: participants.get(i).unwrap().get_last_put_amount()?,
                            })
                        })
                        .collect::<Result<Vec<_>>>()?;

                    return Ok(Some(
                        StakeSOLCommand {
                            items,
                            state: StakeSOLCommandState::Init,
                        }
                        .with_required_accounts(vec![]),
                    ));
                }
            }
            _ => {
                // there are remaining tokens to handle
                if let Some(item) = self.items.first() {
                    match &self.state {
                        StakeSOLCommandState::Init if item.sol_amount > 0 => {
                            let mut command = self.clone();
                            command.state = StakeSOLCommandState::ReadPoolState;

                            let fund_account = ctx.fund_account.load()?;
                            let token = fund_account.get_supported_token(&item.mint)?;
                            match token.pricing_source.into() {
                                TokenPricingSource::SPLStakePool { address }
                                | TokenPricingSource::MarinadeStakePool { address } => {
                                    return Ok(Some(
                                        command.with_required_accounts([(address, false)]),
                                    ));
                                }
                                _ => err!(
                                    errors::ErrorCode::OperationCommandExecutionFailedException
                                )?,
                            }
                        }
                        StakeSOLCommandState::ReadPoolState => {
                            let mut command = self.clone();
                            command.state = StakeSOLCommandState::Stake;

                            let [pool_account, _remaining_accounts @ ..] = accounts else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };

                            let fund_account = ctx.fund_account.load()?;
                            let token = fund_account.get_supported_token(&item.mint)?;
                            let mut required_accounts = vec![
                                (fund_account.get_reserve_account_address()?, true),
                                (
                                    fund_account
                                        .find_supported_token_account_address(&item.mint)?,
                                    true,
                                ),
                            ];

                            required_accounts.extend(match token.pricing_source.into() {
                                TokenPricingSource::SPLStakePool { address } => {
                                    #[cfg(debug_assertions)]
                                    require_keys_eq!(address, pool_account.key());

                                    staking::SPLStakePoolService::find_accounts_to_deposit_sol(
                                        pool_account,
                                    )?
                                }
                                TokenPricingSource::MarinadeStakePool { address } => {
                                    #[cfg(debug_assertions)]
                                    require_keys_eq!(address, pool_account.key());

                                    staking::MarinadeStakePoolService::find_accounts_to_deposit_sol(
                                        pool_account,
                                    )?
                                }
                                _ => err!(
                                    errors::ErrorCode::OperationCommandExecutionFailedException
                                )?,
                            });

                            return Ok(Some(command.with_required_accounts(required_accounts)));
                        }
                        StakeSOLCommandState::Stake => {

                            let token_pricing_source = {
                                let fund_account = ctx.fund_account.load()?;
                                fund_account.get_supported_token(&item.mint)?.pricing_source.clone()
                            };

                            let (
                                pool_token_mint,
                                to_pool_token_account_amount,
                                minted_supported_token_amount,
                                expected_minted_supported_token_amount,
                            ) = match token_pricing_source.into() {
                                TokenPricingSource::SPLStakePool { address } => {
                                    let [fund_reserve_account, fund_supported_token_account, pool_program, pool_account, pool_token_mint, pool_token_program, withdraw_authority, reserve_stake_account, manager_fee_account, remaining_accounts @ ..] =
                                        accounts
                                    else {
                                        err!(ErrorCode::AccountNotEnoughKeys)?
                                    };

                                    #[cfg(debug_assertions)]
                                    require_keys_eq!(address, pool_account.key());

                                    let expected_minted_supported_token_amount =
                                        FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                                            .new_pricing_service(
                                                remaining_accounts.into_iter().cloned(),
                                            )?
                                            .get_sol_amount_as_token(
                                                pool_token_mint.key,
                                                item.sol_amount,
                                            )?;

                                    let fund_account = ctx.fund_account.load()?;
                                    let (
                                        to_pool_token_account_amount,
                                        minted_supported_token_amount,
                                    ) = staking::SPLStakePoolService::new(
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
                                        &fund_account.get_reserve_account_seeds(),
                                        item.sol_amount,
                                    )?;

                                    (
                                        pool_token_mint,
                                        to_pool_token_account_amount,
                                        minted_supported_token_amount,
                                        expected_minted_supported_token_amount,
                                    )
                                }
                                TokenPricingSource::MarinadeStakePool { address } => {
                                    let [fund_reserve_account, fund_supported_token_account, pool_program, pool_account, pool_token_mint, pool_token_program, system_program, liq_pool_sol_leg, liq_pool_token_leg, liq_pool_token_leg_authority, pool_reserve, pool_token_mint_authority, remaining_accounts @ ..] =
                                        accounts
                                    else {
                                        err!(ErrorCode::AccountNotEnoughKeys)?
                                    };

                                    #[cfg(debug_assertions)]
                                    require_keys_eq!(address, pool_account.key());

                                    let expected_minted_supported_token_amount =
                                        FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                                            .new_pricing_service(
                                                remaining_accounts.into_iter().cloned(),
                                            )?
                                            .get_sol_amount_as_token(
                                                pool_token_mint.key,
                                                item.sol_amount,
                                            )?;

                                    let fund_account = ctx.fund_account.load()?;
                                    let (
                                        to_pool_token_account_amount,
                                        minted_supported_token_amount,
                                    ) = staking::MarinadeStakePoolService::new(
                                        pool_program,
                                        pool_account,
                                        pool_token_mint,
                                        pool_token_program,
                                        system_program,
                                    )?
                                    .deposit_sol(
                                        liq_pool_sol_leg,
                                        liq_pool_token_leg,
                                        liq_pool_token_leg_authority,
                                        pool_reserve,
                                        pool_token_mint_authority,
                                        fund_reserve_account,
                                        fund_supported_token_account,
                                        &fund_account.get_reserve_account_seeds(),
                                        item.sol_amount,
                                    )?;

                                    (
                                        pool_token_mint,
                                        to_pool_token_account_amount,
                                        minted_supported_token_amount,
                                        expected_minted_supported_token_amount,
                                    )
                                }
                                _ => err!(
                                    errors::ErrorCode::OperationCommandExecutionFailedException
                                )?,
                            };

                            require_gte!(
                                minted_supported_token_amount,
                                expected_minted_supported_token_amount
                            );

                            let mut fund_account = ctx.fund_account.load_mut()?;
                            fund_account.sol_operation_reserved_amount -= item.sol_amount;

                            let supported_token = fund_account
                                .get_supported_token_mut(pool_token_mint.key)?;
                            supported_token.operation_reserved_amount +=
                                minted_supported_token_amount;

                            require_eq!(
                                supported_token.operation_reserved_amount,
                                to_pool_token_account_amount
                            );
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    }

                    // proceed to next token
                    if self.items.len() > 1 {
                        return Ok(Some(
                            StakeSOLCommand {
                                items: self.items[1..].to_vec(),
                                state: StakeSOLCommandState::Init,
                            }
                            .without_required_accounts(),
                        ));
                    }
                }
            }
        }

        // TODO v0.3/operation: next step after stake sol
        Ok(None)
    }
}
