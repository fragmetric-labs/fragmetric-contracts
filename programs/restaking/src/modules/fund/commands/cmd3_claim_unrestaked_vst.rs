use std::cell::Ref;

use anchor_lang::prelude::*;
use anchor_spl::associated_token;

use crate::errors;
use crate::modules::fund::FundAccount;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::restaking::JitoRestakingVaultService;

use super::{
    DenormalizeNTCommand, FundService, OperationCommandContext, OperationCommandEntry,
    OperationCommandResult, SelfExecutable, FUND_ACCOUNT_MAX_RESTAKING_VAULTS,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct ClaimUnrestakedVSTCommand {
    state: ClaimUnrestakedVSTCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct ClaimUnrestakedVSTCommandItem {
    vault: Pubkey,
    receipt_token_mint: Pubkey,
    supported_token_mint: Pubkey,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum ClaimUnrestakedVSTCommandState {
    #[default]
    New,
    Prepare {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        items: Vec<ClaimUnrestakedVSTCommandItem>,
    },
    Execute {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        items: Vec<ClaimUnrestakedVSTCommandItem>,
    },
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ClaimUnrestakedVSTCommandResult {
    pub vault: Pubkey,
    pub receipt_token_mint: Pubkey,
    pub supported_token_mint: Pubkey,
    pub claimed_supported_token_amount: u64,
    pub operation_reserved_supported_token_amount: u64,
    pub unrestaked_receipt_token_amount: u64,
    pub deducted_receipt_token_fee_amount: u64,
    pub total_unrestaking_receipt_token_amount: u64,
}

impl SelfExecutable for ClaimUnrestakedVSTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let (result, entry) = match &self.state {
            ClaimUnrestakedVSTCommandState::New => self.execute_new(ctx, accounts)?,
            ClaimUnrestakedVSTCommandState::Prepare { items } => {
                self.execute_prepare(ctx, accounts, items)?
            }
            ClaimUnrestakedVSTCommandState::Execute { items } => {
                self.execute_execute(ctx, accounts, items)?
            }
        };

        Ok((
            result,
            entry.or_else(|| Some(DenormalizeNTCommand::default().without_required_accounts())),
        ))
    }
}

#[deny(clippy::wildcard_enum_match_arm)]
impl ClaimUnrestakedVSTCommand {
    #[inline(never)]
    fn execute_new<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        _accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let fund_account = ctx.fund_account.load()?;
        let items = fund_account
            .get_restaking_vaults_iter()
            .filter_map(|restaking_vault| {
                if restaking_vault.receipt_token_operation_receivable_amount > 0 {
                    Some(ClaimUnrestakedVSTCommandItem {
                        vault: restaking_vault.vault,
                        receipt_token_mint: restaking_vault.receipt_token_mint,
                        supported_token_mint: restaking_vault.supported_token_mint,
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok((
            None,
            self.create_prepare_command_with_items(fund_account, items)?,
        ))
    }

    fn create_prepare_command_with_items<'info>(
        &self,
        fund_account: Ref<FundAccount>,
        items: Vec<ClaimUnrestakedVSTCommandItem>,
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
                ClaimUnrestakedVSTCommand {
                    state: ClaimUnrestakedVSTCommandState::Prepare { items },
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
        items: &Vec<ClaimUnrestakedVSTCommandItem>,
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
                let required_accounts = vault_service
                    .find_accounts_to_withdraw()?
                    .chain([
                        (
                            fund_account.find_vault_supported_token_reserve_account_address(
                                vault_account.key,
                            )?,
                            true,
                        ),
                        (fund_account.get_reserve_account_address()?, true),
                    ])
                    .chain(
                        (0..5)
                            .map(|index| {
                                let ticket_base_account =
                                    *FundAccount::find_unrestaking_ticket_account_address(
                                        &ctx.fund_account.key(),
                                        &item.vault,
                                        index,
                                    );
                                let ticket_account = vault_service
                                    .find_withdrawal_ticket_account(&ticket_base_account);
                                let ticket_receipt_token_account =
                                    associated_token::get_associated_token_address_with_program_id(
                                        &ticket_account,
                                        &item.receipt_token_mint,
                                        &anchor_spl::token::ID,
                                    );
                                [(ticket_account, true), (ticket_receipt_token_account, true)]
                            })
                            .flatten(),
                    );

                Ok((
                    None,
                    Some(
                        ClaimUnrestakedVSTCommand {
                            state: ClaimUnrestakedVSTCommandState::Execute {
                                items: items.clone(),
                            },
                        }
                        .with_required_accounts(required_accounts),
                    ),
                ))
            }
            Some(TokenPricingSource::SolvBTCVault { .. }) => {
                // TODO/v0.7.0: deal with solv vault if needed
                Ok((
                    None,
                    Some(
                        ClaimUnrestakedVSTCommand {
                            state: ClaimUnrestakedVSTCommandState::Execute {
                                items: items.clone(),
                            },
                        }
                        .without_required_accounts(),
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
            | Some(TokenPricingSource::PeggedToken { .. })
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
        items: &Vec<ClaimUnrestakedVSTCommandItem>,
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if items.is_empty() {
            return Ok((None, None));
        }

        let item = &items[0];
        let pricing_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
            .new_pricing_service(accounts.iter().copied())?;
        let fund_account = ctx.fund_account.load()?;
        let restaking_vault = fund_account.get_restaking_vault(&item.vault)?;

        let result = match restaking_vault
            .receipt_token_pricing_source
            .try_deserialize()?
        {
            Some(TokenPricingSource::JitoRestakingVault { address }) => {
                require_keys_eq!(address, item.vault);
                let [vault_program, vault_config, vault_account, token_program, system_program, vault_receipt_token_mint, vault_program_fee_receipt_token_account, vault_fee_receipt_token_account, vault_supported_token_reserve_account, fund_vault_supported_token_reserve_account, fund_reserve_account, remaining_accounts @ ..] =
                    accounts
                else {
                    err!(ErrorCode::AccountNotEnoughKeys)?
                };
                let withdrawal_ticket_candidate_accounts = {
                    if remaining_accounts.len() < 10 {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    }
                    &remaining_accounts[..10]
                };

                let vault_service =
                    JitoRestakingVaultService::new(vault_program, vault_config, vault_account)?;

                let claimable_withdrawal_ticket_accounts_list = (0..5)
                    .map(|i| {
                        let ticket = withdrawal_ticket_candidate_accounts[i * 2];
                        let receipt_token_account = withdrawal_ticket_candidate_accounts[i * 2 + 1];

                        Ok(if vault_service.is_claimable_withdrawal_ticket(ticket)? {
                            Some((i, ticket, receipt_token_account))
                        } else {
                            None
                        })
                    })
                    .collect::<Result<Vec<_>>>()?
                    .iter()
                    .flatten()
                    .cloned()
                    .collect::<Vec<_>>();

                if claimable_withdrawal_ticket_accounts_list.len() > 0 {
                    let mut result = ClaimUnrestakedVSTCommandResult {
                        vault: item.vault,
                        receipt_token_mint: item.receipt_token_mint,
                        supported_token_mint: item.supported_token_mint,
                        claimed_supported_token_amount: 0,
                        operation_reserved_supported_token_amount: 0,
                        unrestaked_receipt_token_amount: 0,
                        deducted_receipt_token_fee_amount: 0,
                        total_unrestaking_receipt_token_amount: 0,
                    };
                    let mut last_to_vault_supported_token_account_amount = 0;
                    for (
                        _withdrawal_ticket_index,
                        withdrawal_ticket_account,
                        withdrawal_ticket_receipt_token_account,
                    ) in claimable_withdrawal_ticket_accounts_list
                    {
                        let (
                            to_vault_supported_token_account_amount,
                            unrestaked_receipt_token_amount,
                            claimed_supported_token_amount,
                            deducted_program_fee_receipt_token_amount,
                            deducted_vault_fee_receipt_token_amount,
                            _returned_rent_fee_sol_amount,
                        ) = vault_service.withdraw(
                            token_program,
                            system_program,
                            vault_receipt_token_mint,
                            vault_program_fee_receipt_token_account,
                            vault_fee_receipt_token_account,
                            vault_supported_token_reserve_account,
                            withdrawal_ticket_account,
                            withdrawal_ticket_receipt_token_account,
                            fund_vault_supported_token_reserve_account,
                            fund_reserve_account,
                            &[&ctx.fund_account.load()?.get_reserve_account_seeds()],
                            ctx.operator,
                        )?;

                        require_gte!(
                            fund_reserve_account.lamports(),
                            fund_account.sol.get_total_reserved_amount()
                        );
                        result.claimed_supported_token_amount += claimed_supported_token_amount;
                        result.unrestaked_receipt_token_amount += unrestaked_receipt_token_amount;
                        result.deducted_receipt_token_fee_amount +=
                            deducted_program_fee_receipt_token_amount
                                + deducted_vault_fee_receipt_token_amount;

                        last_to_vault_supported_token_account_amount =
                            to_vault_supported_token_account_amount;
                    }

                    drop(fund_account);
                    let mut fund_account = ctx.fund_account.load_mut()?;

                    let restaking_vault = fund_account.get_restaking_vault_mut(&item.vault)?;
                    restaking_vault.receipt_token_operation_receivable_amount -=
                        result.unrestaked_receipt_token_amount;
                    result.total_unrestaking_receipt_token_amount =
                        restaking_vault.receipt_token_operation_receivable_amount;

                    let deducted_fee_amount_as_sol = pricing_service.get_token_amount_as_sol(
                        &vault_receipt_token_mint.key,
                        result.deducted_receipt_token_fee_amount,
                    )?;
                    match fund_account.get_normalized_token() {
                        Some(normalized_token)
                            if normalized_token.mint == item.supported_token_mint =>
                        {
                            fund_account.sol.operation_receivable_amount +=
                                deducted_fee_amount_as_sol;
                            let normalized_token = fund_account.get_normalized_token_mut().unwrap();
                            normalized_token.operation_reserved_amount +=
                                result.claimed_supported_token_amount;
                            result.operation_reserved_supported_token_amount =
                                normalized_token.operation_reserved_amount;

                            require_gte!(
                                last_to_vault_supported_token_account_amount,
                                normalized_token.operation_reserved_amount
                            );
                        }
                        _ => {
                            let supported_token =
                                fund_account.get_supported_token_mut(&item.supported_token_mint)?;
                            supported_token.token.operation_receivable_amount += pricing_service
                                .get_sol_amount_as_token(
                                    &supported_token.mint,
                                    deducted_fee_amount_as_sol,
                                )?;
                            supported_token.token.operation_reserved_amount +=
                                result.claimed_supported_token_amount;
                            result.operation_reserved_supported_token_amount =
                                supported_token.token.operation_reserved_amount;

                            require_gte!(
                                last_to_vault_supported_token_account_amount,
                                supported_token.token.operation_reserved_amount
                            );
                        }
                    };
                    drop(fund_account);

                    Some(result.into())
                } else {
                    None
                }
            }
            Some(TokenPricingSource::SolvBTCVault { .. }) => {
                // TODO/v0.7.0: deal with solv vault if needed
                None
            }
            // invalid configuration
            Some(TokenPricingSource::SPLStakePool { .. })
            | Some(TokenPricingSource::MarinadeStakePool { .. })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::PeggedToken { .. })
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
            self.create_prepare_command_with_items(fund_account, items)?,
        ))
    }
}
