use anchor_lang::prelude::*;

use super::{
    DelegateVSTCommand, FundService, OperationCommandContext, OperationCommandEntry,
    OperationCommandResult, SelfExecutable, WeightedAllocationParticipant,
    WeightedAllocationStrategy, FUND_ACCOUNT_MAX_RESTAKING_VAULTS,
};
use crate::modules::fund::FUND_ACCOUNT_MAX_SUPPORTED_TOKENS;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::restaking::{JitoRestakingVaultService, SolvBTCVaultService};
use crate::utils::PDASeeds;
use crate::{errors, modules::pricing};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct RestakeVSTCommand {
    state: RestakeVSTCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct RestakeVSTCommandItem {
    vault: Pubkey,
    supported_token_mint: Pubkey,
    allocated_token_amount: u64,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum RestakeVSTCommandState {
    /// Initializes a command with items based on the fund state and strategy.
    #[default]
    New,
    /// Prepares to execute restaking for the first item in the list.
    Prepare {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        items: Vec<RestakeVSTCommandItem>,
    },
    /// Executes restaking for the first item and transitions to the next command,
    /// either preparing the next item or performing a delegation operation.
    Execute {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        items: Vec<RestakeVSTCommandItem>,
    },
}

const RESTAKING_MINIMUM_DEPOSIT_LAMPORTS: u64 = 1_000_000_000;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct RestakeVSTCommandResult {
    pub supported_token_mint: Pubkey,
    pub deposited_supported_token_amount: u64,
    pub deducted_supported_token_fee_amount: u64,
    pub minted_token_amount: u64,
    pub operation_reserved_token_amount: u64,
}

impl SelfExecutable for RestakeVSTCommand {
    fn execute<'a, 'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let mut remaining_items: Option<Vec<RestakeVSTCommandItem>> = None;
        let mut result: Option<OperationCommandResult> = None;

        match &self.state {
            RestakeVSTCommandState::New => {
                let pricing_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                    .new_pricing_service(accounts.into_iter().copied(), true)?;
                let fund_account = ctx.fund_account.load()?;

                // find restakable tokens with their restakable amount among ST and NT
                let mut restakable_token_and_amounts =
                    Vec::with_capacity(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS + 1);
                for supported_token in fund_account.get_supported_tokens_iter() {
                    let supported_token_net_operation_reserved_amount = fund_account
                        .get_asset_net_operation_reserved_amount(
                            Some(supported_token.mint),
                            true,
                            &pricing_service,
                        )?;
                    restakable_token_and_amounts.push((
                        &supported_token.mint,
                        if supported_token_net_operation_reserved_amount > 0 {
                            let supported_token_restaking_reserved_amount =
                                u64::try_from(supported_token_net_operation_reserved_amount)?;
                            supported_token_restaking_reserved_amount
                        } else {
                            0
                        },
                    ));
                }
                if let Some(normalized_token) = fund_account.get_normalized_token() {
                    restakable_token_and_amounts.push((
                        &normalized_token.mint,
                        normalized_token.operation_reserved_amount,
                    ))
                }

                // calculate allocation of tokens for each restaking vaults
                let mut items =
                    Vec::<RestakeVSTCommandItem>::with_capacity(FUND_ACCOUNT_MAX_RESTAKING_VAULTS);
                for (token_mint, token_amount) in restakable_token_and_amounts {
                    let restakable_vaults = fund_account
                        .get_restaking_vaults_iter()
                        .filter(|restaking_vault| {
                            restaking_vault.supported_token_mint == *token_mint
                        })
                        .collect::<Vec<_>>();
                    if restakable_vaults.is_empty() {
                        continue;
                    }

                    let mut strategy =
                        WeightedAllocationStrategy::<FUND_ACCOUNT_MAX_RESTAKING_VAULTS>::new(
                            restakable_vaults
                                .iter()
                                .map(|restaking_vault| {
                                    Ok(WeightedAllocationParticipant::new(
                                        restaking_vault.sol_allocation_weight,
                                        pricing_service.get_token_amount_as_sol(
                                            &restaking_vault.receipt_token_mint,
                                            restaking_vault.receipt_token_operation_reserved_amount,
                                        )?,
                                        restaking_vault.sol_allocation_capacity_amount,
                                    ))
                                })
                                .collect::<Result<Vec<_>>>()?,
                        );

                    strategy
                        .put(pricing_service.get_token_amount_as_sol(&token_mint, token_amount)?)?;

                    for (index, strategy_participant) in
                        strategy.get_participants_iter().enumerate()
                    {
                        let allocated_sol_amount = strategy_participant.get_last_put_amount()?;
                        // try to deposit extra lamports to compensate for flooring errors for each token
                        let allocated_token_amount = (pricing_service
                            .get_sol_amount_as_token(&token_mint, allocated_sol_amount)?
                            + 1)
                        .min(token_amount);

                        if allocated_sol_amount >= RESTAKING_MINIMUM_DEPOSIT_LAMPORTS {
                            let restaking_vault = restakable_vaults.get(index).unwrap();
                            match items
                                .iter_mut()
                                .find(|item| item.vault == restaking_vault.vault)
                            {
                                None => items.push(RestakeVSTCommandItem {
                                    vault: restaking_vault.vault,
                                    supported_token_mint: restaking_vault.supported_token_mint,
                                    allocated_token_amount,
                                }),
                                Some(item) => item.allocated_token_amount += allocated_token_amount,
                            };
                        }
                    }
                }

                if items.len() > 0 {
                    remaining_items = Some(items);
                }
            }
            RestakeVSTCommandState::Prepare { items } => {
                if let Some(item) = items.first() {
                    let fund_account = ctx.fund_account.load()?;
                    let restaking_vault = fund_account.get_restaking_vault(&item.vault)?;

                    match restaking_vault
                        .receipt_token_pricing_source
                        .try_deserialize()?
                    {
                        Some(TokenPricingSource::JitoRestakingVault { address }) => {
                            let [vault_program, vault_config, vault_account, ..] = accounts else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };
                            require_keys_eq!(address, vault_account.key());

                            let required_accounts = JitoRestakingVaultService::new(
                                vault_program,
                                vault_config,
                                vault_account,
                            )?
                            .find_accounts_to_deposit()?
                            .chain([
                                // from_supported_token_account,
                                (
                                    match fund_account.get_normalized_token() {
                                        Some(normalized_token)
                                            if normalized_token.mint
                                                == item.supported_token_mint =>
                                        {
                                            fund_account
                                                .find_normalized_token_reserve_account_address()?
                                        }
                                        _ => fund_account
                                            .find_supported_token_reserve_account_address(
                                                &item.supported_token_mint,
                                            )?,
                                    },
                                    true,
                                ),
                                (
                                    fund_account.find_vault_receipt_token_reserve_account_address(
                                        &restaking_vault.vault,
                                    )?,
                                    true,
                                ),
                                // Jito requires signer to be writable lol
                                (fund_account.get_reserve_account_address()?, true),
                            ]);

                            return Ok((
                                None,
                                Some(
                                    RestakeVSTCommand {
                                        state: RestakeVSTCommandState::Execute {
                                            items: items.clone(),
                                        },
                                    }
                                    .with_required_accounts(required_accounts),
                                ),
                            ));
                        }
                        Some(TokenPricingSource::VirtualVault { .. }) => {
                            remaining_items =
                                Some(items.into_iter().skip(1).copied().collect::<Vec<_>>());
                        }
                        Some(TokenPricingSource::SolvBTCVault { address }) => {
                            let [vault_program, vault_account, ..] = accounts else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };
                            require_keys_eq!(vault_account.key(), address);

                            let required_accounts =
                                SolvBTCVaultService::new(vault_program, vault_account)?
                                    .find_accounts_to_deposit()?
                                    .chain([
                                        (
                                            fund_account
                                                .find_vault_supported_token_reserve_account_address(
                                                    &address,
                                                )?,
                                            true,
                                        ),
                                        (
                                            fund_account
                                                .find_vault_receipt_token_reserve_account_address(
                                                    &restaking_vault.vault,
                                                )?,
                                            true,
                                        ),
                                        (fund_account.get_reserve_account_address()?, false),
                                    ]);

                            return Ok((
                                None,
                                Some(
                                    RestakeVSTCommand {
                                        state: RestakeVSTCommandState::Execute {
                                            items: items.clone(),
                                        },
                                    }
                                    .with_required_accounts(required_accounts),
                                ),
                            ));
                        }
                        // otherwise fails
                        Some(TokenPricingSource::SPLStakePool { .. })
                        | Some(TokenPricingSource::MarinadeStakePool { .. })
                        | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
                        | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. })
                        | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                        | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                        | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                        | Some(TokenPricingSource::PeggedToken { .. })
                        | None => {
                            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                        #[cfg(all(test, not(feature = "idl-build")))]
                        Some(TokenPricingSource::Mock { .. }) => {
                            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                    }
                }
            }
            RestakeVSTCommandState::Execute { items } => {
                if let Some(item) = items.first() {
                    let fund_account = ctx.fund_account.load()?;
                    let restaking_vault = fund_account.get_restaking_vault(&item.vault)?;
                    let receipt_token_pricing_source = restaking_vault
                        .receipt_token_pricing_source
                        .try_deserialize()?;

                    drop(fund_account);

                    match receipt_token_pricing_source {
                        Some(TokenPricingSource::JitoRestakingVault { address }) => {
                            let [vault_program, vault_config, vault_account, token_program, vault_receipt_token_mint, vault_receipt_token_fee_wallet_account, vault_supported_token_reserve_account, from_vault_supported_token_account, to_vault_receipt_token_account, fund_reserve_account, ..] =
                                accounts
                            else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };
                            require_keys_eq!(address, vault_account.key());

                            let mut pricing_service =
                                FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                                    .new_pricing_service(accounts.iter().copied(), false)?;

                            let mut fund_account = ctx.fund_account.load_mut()?;
                            let restaking_vault =
                                fund_account.get_restaking_vault_mut(&item.vault)?;
                            let receipt_token_mint = &restaking_vault.receipt_token_mint;

                            let (
                                supported_token_amount_numerator,
                                receipt_token_amount_denominator,
                            ) = pricing_service
                                .get_vault_supported_token_to_receipt_token_exchange_ratio(
                                    &receipt_token_mint,
                                )?;
                            restaking_vault
                                .update_supported_token_to_receipt_token_exchange_ratio(
                                    supported_token_amount_numerator,
                                    receipt_token_amount_denominator,
                                )?;

                            drop(fund_account);

                            let fund_account = ctx.fund_account.load()?;

                            let vault_service = JitoRestakingVaultService::new(
                                vault_program,
                                vault_config,
                                vault_account,
                            )?;

                            let (
                                to_vault_receipt_token_account_amount,
                                minted_vault_receipt_token_amount,
                                deposited_supported_token_amount,
                                deducted_supported_token_fee_amount,
                            ) = vault_service.deposit(
                                token_program,
                                vault_receipt_token_mint,
                                vault_receipt_token_fee_wallet_account,
                                vault_supported_token_reserve_account,
                                // variant
                                from_vault_supported_token_account,
                                to_vault_receipt_token_account,
                                fund_reserve_account,
                                &[&fund_account.get_reserve_account_seeds()],
                                item.allocated_token_amount,
                            )?;

                            drop(fund_account);

                            FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                                .update_asset_values(&mut pricing_service, false)?;

                            let mut fund_account = ctx.fund_account.load_mut()?;
                            match fund_account.get_normalized_token_mut() {
                                Some(normalized_token)
                                    if normalized_token.mint == item.supported_token_mint =>
                                {
                                    normalized_token.operation_reserved_amount -=
                                        deposited_supported_token_amount;
                                    // accounting receivable of normalized token as SOL
                                    fund_account.sol.operation_receivable_amount += pricing_service
                                        .get_token_amount_as_sol(
                                            &normalized_token.mint,
                                            deducted_supported_token_fee_amount,
                                        )?;
                                }
                                _ => {
                                    let supported_token = fund_account
                                        .get_supported_token_mut(&item.supported_token_mint)?;
                                    supported_token.token.operation_reserved_amount -=
                                        deposited_supported_token_amount;
                                    supported_token.token.operation_receivable_amount +=
                                        deducted_supported_token_fee_amount;
                                }
                            }

                            let restaking_vault =
                                fund_account.get_restaking_vault_mut(&item.vault)?;
                            restaking_vault.receipt_token_operation_reserved_amount +=
                                minted_vault_receipt_token_amount;

                            require_gte!(
                                to_vault_receipt_token_account_amount,
                                restaking_vault.receipt_token_operation_reserved_amount,
                            );

                            result = Some(
                                RestakeVSTCommandResult {
                                    supported_token_mint: item.supported_token_mint,
                                    deposited_supported_token_amount,
                                    deducted_supported_token_fee_amount,
                                    minted_token_amount: minted_vault_receipt_token_amount,
                                    operation_reserved_token_amount: restaking_vault
                                        .receipt_token_operation_reserved_amount,
                                }
                                .into(),
                            );

                            remaining_items =
                                Some(items.into_iter().skip(1).copied().collect::<Vec<_>>());

                            drop(fund_account);
                            FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                                .update_asset_values(&mut pricing_service, true)?;
                        }
                        Some(TokenPricingSource::VirtualVault { .. }) => {
                            remaining_items =
                                Some(items.into_iter().skip(1).copied().collect::<Vec<_>>());
                        }
                        Some(TokenPricingSource::SolvBTCVault { address }) => {
                            let [vault_program, vault_account, vault_receipt_token_mint, vault_supported_token_mint, vault_vault_supported_token_account, token_program, event_authority, fund_vault_supported_token_account, fund_vault_receipt_token_account, fund_reserve, ..] =
                                accounts
                            else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };
                            require_keys_eq!(address, vault_account.key());

                            let mut pricing_service =
                                FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                                    .new_pricing_service(accounts.iter().copied(), false)?;

                            let mut fund_account = ctx.fund_account.load_mut()?;
                            let restaking_vault =
                                fund_account.get_restaking_vault_mut(&item.vault)?;
                            let receipt_token_mint = &restaking_vault.receipt_token_mint;

                            let (
                                supported_token_amount_numerator,
                                receipt_token_amount_denominator,
                            ) = pricing_service
                                .get_vault_supported_token_to_receipt_token_exchange_ratio(
                                    receipt_token_mint,
                                )?;
                            restaking_vault
                                .update_supported_token_to_receipt_token_exchange_ratio(
                                    supported_token_amount_numerator,
                                    receipt_token_amount_denominator,
                                )?;

                            drop(fund_account);

                            let fund_account = ctx.fund_account.load()?;

                            let vault_service =
                                SolvBTCVaultService::new(vault_program, vault_account)?;

                            let (
                                fund_vault_receipt_token_account_amount,
                                minted_vault_receipt_token_amount,
                                deposited_supported_token_amount,
                            ) = vault_service.deposit(
                                vault_receipt_token_mint,
                                vault_supported_token_mint,
                                vault_vault_supported_token_account,
                                token_program,
                                event_authority,
                                ctx.fund_account.as_ref(),
                                &[&fund_account.get_seeds()],
                                fund_vault_receipt_token_account,
                                fund_vault_supported_token_account,
                                fund_reserve,
                                &[&fund_account.get_reserve_account_seeds()],
                                item.allocated_token_amount,
                            )?;
                            drop(fund_account);

                            let mut fund_service =
                                FundService::new(ctx.receipt_token_mint, ctx.fund_account)?;
                            fund_service.update_asset_values(&mut pricing_service, false)?;

                            drop(fund_service);

                            let mut fund_account = ctx.fund_account.load_mut()?;

                            let supported_token =
                                fund_account.get_supported_token_mut(&item.supported_token_mint)?;
                            supported_token.token.operation_reserved_amount -=
                                deposited_supported_token_amount;

                            let restaking_vault =
                                fund_account.get_restaking_vault_mut(&item.vault)?;
                            restaking_vault.receipt_token_operation_reserved_amount +=
                                minted_vault_receipt_token_amount;

                            require_gte!(
                                fund_vault_receipt_token_account_amount,
                                restaking_vault.receipt_token_operation_reserved_amount,
                            );

                            result = Some(
                                RestakeVSTCommandResult {
                                    supported_token_mint: item.supported_token_mint,
                                    deposited_supported_token_amount,
                                    deducted_supported_token_fee_amount: 0,
                                    minted_token_amount: minted_vault_receipt_token_amount,
                                    operation_reserved_token_amount: restaking_vault
                                        .receipt_token_operation_reserved_amount,
                                }
                                .into(),
                            );

                            remaining_items =
                                Some(items.into_iter().skip(1).copied().collect::<Vec<_>>());

                            drop(fund_account);
                            FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                                .update_asset_values(&mut pricing_service, true)?;
                        }
                        // otherwise fails
                        Some(TokenPricingSource::SPLStakePool { .. })
                        | Some(TokenPricingSource::MarinadeStakePool { .. })
                        | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
                        | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. })
                        | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                        | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                        | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                        | Some(TokenPricingSource::PeggedToken { .. })
                        | None => {
                            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                        #[cfg(all(test, not(feature = "idl-build")))]
                        Some(TokenPricingSource::Mock { .. }) => {
                            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                    }
                }
            }
        }

        // transition to next command
        let remaining_items = remaining_items.unwrap_or_default();
        let entry = self
            .create_prepare_command(ctx, remaining_items)?
            .unwrap_or_else(|| DelegateVSTCommand::default().without_required_accounts());

        Ok((result, Some(entry)))
    }
}

impl RestakeVSTCommand {
    fn create_prepare_command(
        &self,
        ctx: &OperationCommandContext,
        remaining_items: Vec<RestakeVSTCommandItem>,
    ) -> Result<Option<OperationCommandEntry>> {
        if remaining_items.len() == 0 {
            return Ok(None);
        }

        let pricing_source = ctx
            .fund_account
            .load()?
            .get_restaking_vault(&remaining_items[0].vault)?
            .receipt_token_pricing_source
            .try_deserialize()?;

        let command = RestakeVSTCommand {
            state: RestakeVSTCommandState::Prepare {
                items: remaining_items,
            },
        };

        let entry = match pricing_source {
            Some(TokenPricingSource::JitoRestakingVault { address }) => {
                let required_accounts = JitoRestakingVaultService::find_accounts_to_new(address)?;
                command.with_required_accounts(required_accounts)
            }
            Some(TokenPricingSource::SolvBTCVault { address }) => {
                let required_accounts = SolvBTCVaultService::find_accounts_to_new(address)?;
                command.with_required_accounts(required_accounts)
            }
            Some(TokenPricingSource::VirtualVault { .. }) => command.without_required_accounts(),
            // otherwise fails
            Some(TokenPricingSource::SPLStakePool { .. })
            | Some(TokenPricingSource::MarinadeStakePool { .. })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | Some(TokenPricingSource::PeggedToken { .. })
            | None => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        };

        Ok(Some(entry))
    }
}
