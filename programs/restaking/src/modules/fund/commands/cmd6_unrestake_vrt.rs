use anchor_lang::prelude::*;
use anchor_spl::associated_token;
use anchor_spl::associated_token::{
    get_associated_token_address, get_associated_token_address_with_program_id,
    spl_associated_token_account,
};
use std::cell::Ref;
use std::iter;
use std::ops::Neg;

use crate::modules::normalization::NormalizedTokenPoolAccount;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::restaking::JitoRestakingVaultService;
use crate::modules::staking::MarinadeStakePoolService;
use crate::utils::{AccountInfoExt, PDASeeds};
use crate::{errors, utils};

use super::{
    ClaimUnstakedSOLCommand, FundAccount, FundService, OperationCommand, OperationCommandContext,
    OperationCommandEntry, OperationCommandResult, SelfExecutable, UndelegateVSTCommand,
    UnstakeLSTCommandItem, WeightedAllocationParticipant, WeightedAllocationStrategy,
    FUND_ACCOUNT_MAX_RESTAKING_VAULTS, FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct UnrestakeVRTCommand {
    state: UnrestakeVRTCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct UnrestakeVSTCommandItem {
    vault: Pubkey,
    receipt_token_mint: Pubkey,
    supported_token_mint: Pubkey,
    allocated_receipt_token_amount: u64,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum UnrestakeVRTCommandState {
    #[default]
    New,
    Prepare {
        #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
        items: Vec<UnrestakeVSTCommandItem>,
    },
    Execute {
        #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
        items: Vec<UnrestakeVSTCommandItem>,
    },
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct UnrestakeVRTCommandResult {
    pub vault: Pubkey,
    pub token_mint: Pubkey,
    pub unrestaking_token_amount: u64,
    pub total_unrestaking_token_amount: u64,
    pub operation_reserved_token_amount: u64,
}

#[deny(clippy::wildcard_enum_match_arm)]
impl SelfExecutable for UnrestakeVRTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let (result, entry) = match &self.state {
            UnrestakeVRTCommandState::New => self.execute_new(ctx, accounts)?,
            UnrestakeVRTCommandState::Prepare { items } => {
                self.execute_prepare(ctx, accounts, items)?
            }
            UnrestakeVRTCommandState::Execute { items } => {
                self.execute_execute(ctx, accounts, items)?
            }
        };

        Ok((
            result,
            entry.or_else(|| Some(ClaimUnstakedSOLCommand::default().without_required_accounts())),
        ))
    }
}

#[deny(clippy::wildcard_enum_match_arm)]
impl UnrestakeVRTCommand {
    #[inline(never)]
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

        // calculate additionally required unstaking amount for each supported tokens
        let unstaking_obligated_amount_as_sol =
            fund_account.get_total_unstaking_obligated_amount_as_sol(&pricing_service)?;

        let mut unstaking_strategy =
            WeightedAllocationStrategy::<FUND_ACCOUNT_MAX_SUPPORTED_TOKENS>::new(
                fund_account
                    .get_supported_tokens_iter()
                    .map(|supported_token| {
                        Ok(WeightedAllocationParticipant::new(
                            supported_token.sol_allocation_weight,
                            match supported_token.pricing_source.try_deserialize()? {
                                // stakable tokens
                                Some(TokenPricingSource::SPLStakePool { .. })
                                | Some(TokenPricingSource::MarinadeStakePool { .. })
                                | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool {
                                    ..
                                }) => pricing_service.get_token_amount_as_sol(
                                    &supported_token.mint,
                                    u64::try_from(
                                        fund_account
                                            .get_asset_net_operation_reserved_amount(
                                                Some(supported_token.mint),
                                                false,
                                                &pricing_service,
                                            )?
                                            .max(0),
                                    )? + normalized_token_pool_account
                                        .get_supported_token(&supported_token.mint)
                                        .map(|t| t.locked_amount)
                                        .unwrap_or_default(),
                                )?,
                                // not stakable tokens
                                Some(TokenPricingSource::OrcaDEXLiquidityPool { .. }) => 0,
                                // invalid configuration
                                Some(TokenPricingSource::FragmetricNormalizedTokenPool {
                                    ..
                                })
                                | Some(TokenPricingSource::JitoRestakingVault { .. })
                                | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                                | None => err!(
                                    errors::ErrorCode::FundOperationCommandExecutionFailedException
                                )?,
                                #[cfg(all(test, not(feature = "idl-build")))]
                                Some(TokenPricingSource::Mock { .. }) => err!(
                                    errors::ErrorCode::FundOperationCommandExecutionFailedException
                                )?,
                            },
                            supported_token.sol_allocation_capacity_amount,
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?,
            );
        let remaining = unstaking_strategy.cut_greedy(unstaking_obligated_amount_as_sol)?;

        // calculate required token amount for each supported tokens' withdrawal obligation
        let mut items = Vec::<UnrestakeVSTCommandItem>::with_capacity(
            FUND_ACCOUNT_MAX_RESTAKING_VAULTS as usize,
        );
        for restaking_vault in fund_account.get_restaking_vaults_iter() {
            items.push(UnrestakeVSTCommandItem {
                vault: restaking_vault.vault,
                receipt_token_mint: restaking_vault.receipt_token_mint,
                supported_token_mint: restaking_vault.supported_token_mint,
                allocated_receipt_token_amount: 0,
            });
        }

        for (supported_token_index, supported_token) in
            fund_account.get_supported_tokens_iter().enumerate()
        {
            let unstaking_obligated_amount_as_sol = unstaking_strategy
                .get_participant_by_index(supported_token_index)?
                .get_last_cut_amount()?;
            let unrestaking_obligated_amount_as_sol = pricing_service.get_token_amount_as_sol(
                &supported_token.mint,
                u64::try_from(
                    fund_account
                        .get_asset_net_operation_reserved_amount(
                            Some(supported_token.mint),
                            true,
                            &pricing_service,
                        )?
                        .min(0)
                        .neg(),
                )?,
            )? + unstaking_obligated_amount_as_sol;

            // it assumes there won't be no more than two duplicate vaults for a same token including normalized token.
            let mut unrestaking_strategy = WeightedAllocationStrategy::<4>::new(
                fund_account
                    .get_restaking_vaults_iter()
                    .enumerate()
                    .map(|(restaking_vault_index, restaking_vault)| {
                        Ok(
                            if restaking_vault.supported_token_mint == supported_token.mint {
                                // it is a vault for this supported token
                                Some(WeightedAllocationParticipant::new(
                                    restaking_vault.sol_allocation_weight,
                                    pricing_service.get_token_amount_as_sol(
                                        &restaking_vault.receipt_token_mint,
                                        // here accounts for previously allocated amount
                                        restaking_vault.receipt_token_operation_reserved_amount
                                            - items[restaking_vault_index]
                                                .allocated_receipt_token_amount,
                                    )?,
                                    restaking_vault.sol_allocation_capacity_amount,
                                ))
                            } else if normalized_token_pool_account
                                .has_supported_token(&supported_token.mint)
                            {
                                // it is a normalized token vault and this supported token belongs to the normalized token pool
                                Some(WeightedAllocationParticipant::new(
                                    restaking_vault.sol_allocation_weight,
                                    // those locked supported tokens in normalized token pool are mutually exclusive,
                                    // so we can keep calculating without accounting for previously allocated amount
                                    utils::get_proportional_amount(
                                        pricing_service.get_token_amount_as_sol(
                                            &restaking_vault.receipt_token_mint,
                                            restaking_vault.receipt_token_operation_reserved_amount,
                                        )?,
                                        pricing_service.get_token_amount_as_sol(
                                            &supported_token.mint,
                                            normalized_token_pool_account
                                                .get_supported_token(&supported_token.mint)?
                                                .locked_amount,
                                        )?,
                                        pricing_service.get_token_amount_as_sol(
                                            &normalized_token_pool_account.normalized_token_mint,
                                            normalized_token_pool_account
                                                .normalized_token_supply_amount,
                                        )?,
                                    )?,
                                    restaking_vault.sol_allocation_capacity_amount,
                                ))
                            } else {
                                None
                            },
                        )
                    })
                    .collect::<Result<Vec<_>>>()?
                    .iter()
                    .flatten()
                    .copied()
                    .collect::<Vec<_>>(),
            );

            unrestaking_strategy.cut_greedy(unrestaking_obligated_amount_as_sol)?;
            for (index, p) in unrestaking_strategy.get_participants_iter().enumerate() {
                let item = &mut items[index];
                item.allocated_receipt_token_amount += pricing_service
                    .get_sol_amount_as_token(&item.receipt_token_mint, p.get_last_cut_amount()?)?;
            }
        }

        items = items
            .iter()
            .filter(|item| item.allocated_receipt_token_amount > 0)
            .copied()
            .collect();

        Ok((
            None,
            self.create_prepare_command_with_items(fund_account, &items)?,
        ))
    }

    fn create_prepare_command_with_items<'info>(
        &self,
        fund_account: Ref<FundAccount>,
        items: &Vec<UnrestakeVSTCommandItem>,
    ) -> Result<Option<OperationCommandEntry>> {
        Ok(if items.len() > 0 {
            let required_accounts = match fund_account
                .get_restaking_vault(&items[0].vault)?
                .receipt_token_pricing_source
                .try_deserialize()?
            {
                Some(TokenPricingSource::JitoRestakingVault { address }) => {
                    JitoRestakingVaultService::find_accounts_to_new(address)?
                }
                _ => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
            };
            Some(
                UnrestakeVRTCommand {
                    state: UnrestakeVRTCommandState::Prepare {
                        items: items.clone(),
                    },
                }
                .with_required_accounts(required_accounts),
            )
        } else {
            None
        })
    }

    fn execute_prepare<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        items: &Vec<UnrestakeVSTCommandItem>,
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if items.is_empty() {
            return Ok((None, None));
        }
        let item = &items[0];
        let fund_account = ctx.fund_account.load()?;
        let restaking_vault = fund_account.get_restaking_vault(&item.vault)?;

        match restaking_vault
            .receipt_token_pricing_source
            .try_deserialize()?
        {
            Some(TokenPricingSource::JitoRestakingVault { address }) => {
                require_keys_eq!(address, item.vault);
                let [vault_program, vault_config, vault_account, _remaining_accounts @ ..] =
                    accounts
                else {
                    err!(ErrorCode::AccountNotEnoughKeys)?
                };

                let vault_service =
                    JitoRestakingVaultService::new(vault_program, vault_config, vault_account)?;
                let mut required_accounts = vault_service.find_accounts_to_request_withdraw()?;
                required_accounts.extend(vec![
                    (
                        fund_account
                            .find_vault_receipt_token_reserve_account_address(vault_account.key)?,
                        true,
                    ),
                    (fund_account.get_reserve_account_address()?, true),
                ]);
                required_accounts.extend(
                    (0..5)
                        .map(|index| {
                            let withdrawal_ticket_authority =
                                *FundAccount::find_unrestaking_ticket_account_address(
                                    &ctx.fund_account.key(),
                                    &item.vault,
                                    index,
                                );
                            let withdrawal_ticket = vault_service
                                .find_withdrawal_ticket_account(&withdrawal_ticket_authority);
                            let withdrawal_ticket_receipt_token_account =
                                associated_token::get_associated_token_address_with_program_id(
                                    &withdrawal_ticket,
                                    &item.receipt_token_mint,
                                    &anchor_spl::token::ID,
                                );
                            [
                                (withdrawal_ticket, true),
                                (withdrawal_ticket_receipt_token_account, true),
                                (withdrawal_ticket_authority, false),
                            ]
                        })
                        .flatten(),
                );

                Ok((
                    None,
                    Some(
                        UnrestakeVRTCommand {
                            state: UnrestakeVRTCommandState::Execute {
                                items: items.clone(),
                            },
                        }
                        .with_required_accounts(required_accounts),
                    ),
                ))
            }
            // invalid configuration
            Some(TokenPricingSource::SPLStakePool { .. })
            | Some(TokenPricingSource::MarinadeStakePool { .. })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | None => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        }
    }

    fn execute_execute<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        items: &Vec<UnrestakeVSTCommandItem>,
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if items.is_empty() {
            return Ok((None, None));
        }

        let item = &items[0];
        let fund_account = ctx.fund_account.load()?;
        let restaking_vault = fund_account.get_restaking_vault(&item.vault)?;

        let result = match restaking_vault
            .receipt_token_pricing_source
            .try_deserialize()?
        {
            Some(TokenPricingSource::JitoRestakingVault { address }) => {
                require_keys_eq!(address, item.vault);
                let [vault_program, vault_config, vault_account, token_program, associated_token, system_program, vault_receipt_token_mint, vault_receipt_token_reserve_account, fund_vault_receipt_token_reserve_account, fund_reserve_account, remaining_accounts @ ..] =
                    accounts
                else {
                    err!(ErrorCode::AccountNotEnoughKeys)?
                };
                let withdrawal_ticket_candidate_accounts = {
                    if remaining_accounts.len() < 15 {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    }
                    &remaining_accounts[..15]
                };
                let withdrawal_ticket_accounts = (0..5).find_map(|i| {
                    let ticket = withdrawal_ticket_candidate_accounts[i * 3];
                    let receipt_token_account = withdrawal_ticket_candidate_accounts[i * 3 + 1];
                    let authority = withdrawal_ticket_candidate_accounts[i * 3 + 2];
                    if !ticket.is_initialized() {
                        Some((i, ticket, receipt_token_account, authority))
                    } else {
                        None
                    }
                });

                if let Some((
                    withdrawal_ticket_index,
                    withdrawal_ticket_account,
                    withdrawal_ticket_receipt_token_account,
                    withdrawal_ticket_authority_account,
                )) = withdrawal_ticket_accounts
                {
                    let vault_service =
                        JitoRestakingVaultService::new(vault_program, vault_config, vault_account)?;
                    let (
                        from_vault_receipt_token_account_amount,
                        enqueued_vault_receipt_token_amount,
                    ) = vault_service.request_withdraw(
                        token_program,
                        associated_token,
                        system_program,
                        vault_receipt_token_mint,
                        vault_receipt_token_reserve_account,
                        fund_vault_receipt_token_reserve_account,
                        withdrawal_ticket_account,
                        withdrawal_ticket_receipt_token_account,
                        withdrawal_ticket_authority_account,
                        ctx.operator,
                        &[],
                        fund_reserve_account,
                        &[
                            &fund_account.get_reserve_account_seeds(),
                            &FundAccount::find_unrestaking_ticket_account_address(
                                &ctx.fund_account.key(),
                                &item.vault,
                                withdrawal_ticket_index as u8,
                            )
                            .get_seeds(),
                        ],
                        item.allocated_receipt_token_amount,
                    )?;

                    require_gte!(
                        fund_reserve_account.lamports(),
                        fund_account.sol.get_total_reserved_amount()
                    );
                    drop(fund_account);

                    let mut fund_account = ctx.fund_account.load_mut()?;
                    let restaking_vault = fund_account.get_restaking_vault_mut(&item.vault)?;
                    restaking_vault.receipt_token_operation_reserved_amount -=
                        enqueued_vault_receipt_token_amount;
                    restaking_vault.receipt_token_operation_receivable_amount +=
                        enqueued_vault_receipt_token_amount;
                    require_gte!(
                        from_vault_receipt_token_account_amount,
                        restaking_vault.receipt_token_operation_reserved_amount
                    );

                    Some(
                        UnrestakeVRTCommandResult {
                            vault: item.vault,
                            token_mint: item.receipt_token_mint,
                            unrestaking_token_amount: enqueued_vault_receipt_token_amount,
                            total_unrestaking_token_amount: restaking_vault
                                .receipt_token_operation_receivable_amount,
                            operation_reserved_token_amount: restaking_vault
                                .receipt_token_operation_reserved_amount,
                        }
                        .into(),
                    )
                } else {
                    None
                }
            }
            // invalid configuration
            Some(TokenPricingSource::SPLStakePool { .. })
            | Some(TokenPricingSource::MarinadeStakePool { .. })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | None => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        };

        let items = items.iter().skip(1).copied().collect::<Vec<_>>();
        let fund_account = ctx.fund_account.load()?;
        Ok((
            result,
            self.create_prepare_command_with_items(fund_account, &items)?,
        ))
    }
}
