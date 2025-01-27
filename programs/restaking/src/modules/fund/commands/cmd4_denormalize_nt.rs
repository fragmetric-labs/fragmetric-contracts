use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};
use anchor_spl::{token::Token, token_interface::TokenInterface};
use std::ops::Neg;

use super::{
    fund_account, FundService, OperationCommand, OperationCommandContext, OperationCommandEntry,
    OperationCommandResult, SelfExecutable, UndelegateVSTCommand,
    FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
};
use crate::modules::fund::commands::OperationCommand::UndelegateVST;
use crate::modules::fund::{WeightedAllocationParticipant, WeightedAllocationStrategy};
use crate::{
    errors,
    modules::{
        normalization::{NormalizedTokenPoolAccount, NormalizedTokenPoolService},
        pricing::TokenPricingSource,
    },
    utils,
};
use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct DenormalizeNTCommand {
    state: DenormalizeNTCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct DenormalizeNTCommandItem {
    supported_token_mint: Pubkey,
    allocated_normalized_token_amount: u64,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum DenormalizeNTCommandState {
    #[default]
    New,
    Prepare {
        #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
        items: Vec<DenormalizeNTCommandItem>,
    },
    Execute {
        #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
        items: Vec<DenormalizeNTCommandItem>,
    },
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct DenormalizeNTCommandResult {
    pub supported_token_mint: Pubkey,
    pub burnt_normalized_token_amount: u64,
    pub operation_reserved_normalized_token_amount: u64,
    pub denormalized_supported_token_amount: u64,
    pub operation_reserved_supported_token_amount: u64,
}

impl SelfExecutable for DenormalizeNTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let (result, entry) = match &self.state {
            DenormalizeNTCommandState::New => self.execute_new(ctx, accounts)?,
            DenormalizeNTCommandState::Prepare { items } => {
                self.execute_prepare(ctx, accounts, items.clone(), None)?
            }
            DenormalizeNTCommandState::Execute { items } => {
                self.execute_execute(ctx, accounts, items)?
            }
        };

        Ok((
            result,
            entry.or_else(|| Some(UndelegateVSTCommand::default().without_required_accounts())),
        ))
    }
}

impl DenormalizeNTCommand {
    fn execute_new<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let pricing_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
            .new_pricing_service(accounts.iter().copied())?;
        let fund_account = ctx.fund_account.load()?;

        let normalized_token = fund_account.get_normalized_token();
        if normalized_token.is_none() {
            return Ok((None, None));
        }
        let normalized_token = normalized_token.unwrap();
        let total_normalized_token_reserved_amount = normalized_token.operation_reserved_amount;
        if total_normalized_token_reserved_amount == 0 {
            return Ok((None, None));
        }

        let normalized_token_pool_account = fund_account
            .get_normalized_token_pool_address()
            .and_then(|address| {
                accounts
                    .iter()
                    .find(|account| account.key() == address)
                    .copied()
            })
            .ok_or_else(|| error!(errors::ErrorCode::FundOperationCommandExecutionFailedException))
            .and_then(Account::<NormalizedTokenPoolAccount>::try_from)?;

        // allocate items with withdrawal obligated token amounts
        let supported_tokens = fund_account
            .get_supported_tokens_iter()
            .filter(|supported_token| {
                normalized_token_pool_account.has_supported_token(&supported_token.mint)
            })
            .collect::<Vec<_>>();
        let mut items =
            Vec::<DenormalizeNTCommandItem>::with_capacity(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS);

        let mut remaining_normalized_token_reserved_amount = total_normalized_token_reserved_amount;
        for supported_token in &supported_tokens {
            let withdrawal_obligated_supported_token_amount = u64::try_from(
                fund_account
                    .get_asset_net_operation_reserved_amount(
                        Some(supported_token.mint),
                        false,
                        &pricing_service,
                    )?
                    .min(0)
                    .neg(),
            )?;

            let allocated_supported_token_amount = normalized_token_pool_account
                .get_supported_token(&supported_token.mint)?
                .locked_amount
                .min(withdrawal_obligated_supported_token_amount);

            let allocated_normalized_token_amount = if allocated_supported_token_amount > 0 {
                pricing_service
                    .get_token_amount_as_asset(
                        &supported_token.mint,
                        allocated_supported_token_amount,
                        Some(&normalized_token_pool_account.normalized_token_mint),
                    )?
                    .min(remaining_normalized_token_reserved_amount)
            } else {
                0
            };

            remaining_normalized_token_reserved_amount -= allocated_normalized_token_amount;

            items.push(DenormalizeNTCommandItem {
                supported_token_mint: supported_token.mint,
                allocated_normalized_token_amount,
            });
        }

        // allocate remaining amounts
        if remaining_normalized_token_reserved_amount > 0 {
            let mut remaining_strategy =
                WeightedAllocationStrategy::<FUND_ACCOUNT_MAX_SUPPORTED_TOKENS>::new(
                    supported_tokens
                        .clone()
                        .into_iter()
                        .enumerate()
                        .map(|(index, supported_token)| {
                            let normalized_supported_token = normalized_token_pool_account
                                .get_supported_token(&supported_token.mint)?;

                            // here, conversion from NT to SOL is not required as we just cut the remaining amount following the weights
                            Ok(WeightedAllocationParticipant::new(
                                supported_token.sol_allocation_weight,
                                pricing_service.get_token_amount_as_asset(
                                    &supported_token.mint,
                                    normalized_supported_token.locked_amount,
                                    Some(&normalized_token.mint),
                                )? - items[index].allocated_normalized_token_amount,
                                0,
                            ))
                        })
                        .collect::<Result<Vec<_>>>()?,
                );
            let remains =
                remaining_strategy.cut_greedy(remaining_normalized_token_reserved_amount)?;
            require_gte!(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS as u64, remains);

            for (index, _) in supported_tokens.iter().enumerate() {
                items[index].allocated_normalized_token_amount +=
                    remaining_strategy.get_participant_last_cut_amount_by_index(index)?;
            }
        }

        let items = items
            .iter()
            .filter(|item| item.allocated_normalized_token_amount > 0)
            .copied()
            .collect::<Vec<_>>();

        // nothing to denormalize
        if items.is_empty() {
            return Ok((None, None));
        }

        drop(fund_account);
        self.execute_prepare(ctx, accounts, items, None)
    }

    fn execute_prepare<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        items: Vec<DenormalizeNTCommandItem>,
        previous_execution_result: Option<OperationCommandResult>,
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if items.is_empty() {
            return Ok((previous_execution_result, None));
        }
        let item = &items[0];

        let fund_account = ctx.fund_account.load()?;
        let supported_token = fund_account.get_supported_token(&item.supported_token_mint)?;
        let normalized_token_pool_account_info = fund_account
            .get_normalized_token_pool_address()
            .and_then(|address| {
                accounts
                    .iter()
                    .find(|account| account.key() == address)
                    .copied()
            })
            .ok_or_else(|| {
                error!(errors::ErrorCode::FundOperationCommandExecutionFailedException)
            })?;

        let accounts_to_denormalize_supported_token =
            NormalizedTokenPoolService::find_accounts_to_denormalize_supported_token(
                normalized_token_pool_account_info,
                &supported_token.mint,
            )?;
        let fund_supported_token_reserve_account =
            fund_account.find_supported_token_reserve_account_address(&supported_token.mint)?;
        let fund_normalized_token_reserve_account =
            fund_account.find_normalized_token_reserve_account_address()?;
        let fund_reserve_account = fund_account.get_reserve_account_address()?;

        let required_accounts = accounts_to_denormalize_supported_token.chain([
            // to_supported_token_account
            (fund_supported_token_reserve_account, true),
            // from_normalized_token_account
            (fund_normalized_token_reserve_account, true),
            // from_normalized_token_account_signer
            (fund_reserve_account, false),
        ]);

        let command = Self {
            state: DenormalizeNTCommandState::Execute { items },
        }
        .with_required_accounts(required_accounts);

        Ok((previous_execution_result, Some(command)))
    }

    fn execute_execute<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        items: &[DenormalizeNTCommandItem],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if items.is_empty() {
            return Ok((None, None));
        }
        let item = items[0];

        let pricing_source = ctx
            .fund_account
            .load()?
            .get_normalized_token()
            .and_then(|normalized_token| normalized_token.pricing_source.try_deserialize().ok())
            .flatten();

        match pricing_source {
            Some(TokenPricingSource::FragmetricNormalizedTokenPool { address }) => {
                let [normalized_token_pool_account, normalized_token_mint, normalized_token_program, supported_token_mint, supported_token_program, supported_token_reserve_account, fund_supported_token_reserve_account, fund_normalized_token_reserve_account, fund_reserve_account, pricing_sources @ ..] =
                    accounts
                else {
                    err!(ErrorCode::AccountNotEnoughKeys)?
                };
                require_keys_eq!(normalized_token_pool_account.key(), address);

                let mut pricing_service =
                    FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                        .new_pricing_service(pricing_sources.iter().cloned())?;

                let mut normalized_token_pool_account =
                    Account::<NormalizedTokenPoolAccount>::try_from(normalized_token_pool_account)?;
                let mut normalized_token_mint =
                    InterfaceAccount::<Mint>::try_from(normalized_token_mint)?;
                let normalized_token_program =
                    Program::<Token>::try_from(*normalized_token_program)?;
                let supported_token_mint =
                    InterfaceAccount::<Mint>::try_from(supported_token_mint)?;
                let supported_token_program =
                    Interface::<TokenInterface>::try_from(*supported_token_program)?;
                let supported_token_reserve_account =
                    InterfaceAccount::<TokenAccount>::try_from(supported_token_reserve_account)?;
                let mut fund_supported_token_reserve_account =
                    InterfaceAccount::<TokenAccount>::try_from(
                        fund_supported_token_reserve_account,
                    )?;
                let fund_normalized_token_reserve_account =
                    InterfaceAccount::<TokenAccount>::try_from(
                        fund_normalized_token_reserve_account,
                    )?;

                let (to_supported_token_account_amount, denormalized_supported_token_amount) =
                    NormalizedTokenPoolService::new(
                        &mut normalized_token_pool_account,
                        &mut normalized_token_mint,
                        &normalized_token_program,
                    )?
                    .denormalize_supported_token(
                        &supported_token_mint,
                        &supported_token_program,
                        &supported_token_reserve_account,
                        &mut fund_supported_token_reserve_account,
                        &fund_normalized_token_reserve_account,
                        fund_reserve_account,
                        &[&ctx.fund_account.load()?.get_reserve_account_seeds()],
                        item.allocated_normalized_token_amount,
                        &mut pricing_service,
                    )?;

                // validation
                let expected_denormalized_supported_token_amount = pricing_service
                    .get_sol_amount_as_token(
                        &supported_token_mint.key(),
                        pricing_service.get_token_amount_as_sol(
                            &normalized_token_mint.key(),
                            item.allocated_normalized_token_amount,
                        )?,
                    )?;
                require_gte!(
                    expected_denormalized_supported_token_amount,
                    denormalized_supported_token_amount
                );

                // update fund account
                let mut fund_account = ctx.fund_account.load_mut()?;

                let supported_token =
                    fund_account.get_supported_token_mut(&item.supported_token_mint)?;
                supported_token.token.operation_reserved_amount +=
                    denormalized_supported_token_amount;
                let operation_reserved_supported_token_amount =
                    supported_token.token.operation_reserved_amount;

                require_gte!(
                    to_supported_token_account_amount,
                    supported_token.token.get_total_reserved_amount()
                );

                let normalized_token = fund_account.get_normalized_token_mut().unwrap();
                normalized_token.operation_reserved_amount -=
                    item.allocated_normalized_token_amount;

                let result = Some(
                    DenormalizeNTCommandResult {
                        supported_token_mint: item.supported_token_mint,
                        burnt_normalized_token_amount: item.allocated_normalized_token_amount,
                        operation_reserved_normalized_token_amount: normalized_token
                            .operation_reserved_amount,
                        denormalized_supported_token_amount,
                        operation_reserved_supported_token_amount,
                    }
                    .into(),
                );

                drop(fund_account);
                self.execute_prepare(ctx, accounts, items[1..].to_vec(), result)
            }
            // otherwise fails
            Some(TokenPricingSource::SPLStakePool { .. })
            | Some(TokenPricingSource::MarinadeStakePool { .. })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | None => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        }
    }
}
