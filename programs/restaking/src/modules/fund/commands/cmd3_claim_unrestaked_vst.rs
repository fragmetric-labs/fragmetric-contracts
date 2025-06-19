use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::restaking::{JitoRestakingVaultService, SolvBTCVaultService};
use crate::utils::PDASeeds;

use super::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct ClaimUnrestakedVSTCommand {
    state: ClaimUnrestakedVSTCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum ClaimUnrestakedVSTCommandState {
    #[default]
    New,
    Prepare {
        vault: Pubkey,
    },
    Execute {
        vault: Pubkey,
    },
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct ClaimUnrestakedVSTCommandResult {
    pub vault: Pubkey,
    pub receipt_token_mint: Pubkey,
    pub total_unrestaking_receipt_token_amount: u64,
    pub unrestaked_receipt_token_amount: u64,
    pub deducted_receipt_token_fee_amount: u64,
    pub supported_token_mint: Pubkey,
    pub claimed_supported_token_amount: u64,
    pub transferred_supported_token_revenue_amount: u64,
    pub offsetted_supported_token_receivable_amount: u64,
    pub offsetted_asset_receivables: Vec<ClaimUnrestakedVSTCommandResultAssetReceivable>,
    pub operation_reserved_supported_token_amount: u64,
    pub operation_receivable_supported_token_amount: u64,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct ClaimUnrestakedVSTCommandResultAssetReceivable {
    asset_token_mint: Option<Pubkey>,
    asset_amount: u64,
}

impl SelfExecutable for ClaimUnrestakedVSTCommand {
    fn execute<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> ExecutionResult {
        let (result, entry) = match &self.state {
            ClaimUnrestakedVSTCommandState::New => self.execute_new(ctx, None, None)?,
            ClaimUnrestakedVSTCommandState::Prepare { vault } => {
                self.execute_prepare(ctx, accounts, vault)?
            }
            ClaimUnrestakedVSTCommandState::Execute { vault } => {
                self.execute_execute(ctx, accounts, vault)?
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
    fn execute_new(
        &self,
        ctx: &OperationCommandContext,
        previous_vault: Option<&Pubkey>,
        previous_execution_result: Option<OperationCommandResult>,
    ) -> ExecutionResult {
        let fund_account = ctx.fund_account.load()?;
        let Some(vault) = ({
            let mut vaults_iter = fund_account
                .get_restaking_vaults_iter()
                .map(|restaking_vault| &restaking_vault.vault);
            if let Some(previous_vault) = previous_vault {
                vaults_iter
                    .skip_while(|vault| *vault != previous_vault)
                    .nth(1)
            } else {
                vaults_iter.next()
            }
        }) else {
            // fallback: cmd4: denormalize_nt
            return Ok((previous_execution_result, None));
        };

        let restaking_vault = fund_account.get_restaking_vault(vault)?;
        let receipt_token_pricing_source = restaking_vault
            .receipt_token_pricing_source
            .try_deserialize()?;
        let Some(entry) = (|| {
            Result::Ok(Some(match receipt_token_pricing_source {
                Some(TokenPricingSource::JitoRestakingVault { address }) => {
                    if restaking_vault.receipt_token_operation_receivable_amount == 0 {
                        return Ok(None);
                    }

                    let required_accounts =
                        JitoRestakingVaultService::find_accounts_to_new(address)?;
                    let command = ClaimUnrestakedVSTCommand {
                        state: ClaimUnrestakedVSTCommandState::Prepare { vault: *vault },
                    };
                    command.with_required_accounts(required_accounts)
                }
                Some(TokenPricingSource::SolvBTCVault { address }) => {
                    let required_accounts = SolvBTCVaultService::find_accounts_to_new(address)?;
                    let command = ClaimUnrestakedVSTCommand {
                        state: ClaimUnrestakedVSTCommandState::Prepare { vault: *vault },
                    };
                    command.with_required_accounts(required_accounts)
                }
                // no unrestaking on virtual vault
                Some(TokenPricingSource::VirtualVault { .. }) => return Ok(None),
                // otherwise fails
                Some(TokenPricingSource::SPLStakePool { .. })
                | Some(TokenPricingSource::MarinadeStakePool { .. })
                | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
                | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. })
                | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                | Some(TokenPricingSource::PeggedToken { .. })
                | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
                #[cfg(all(test, not(feature = "idl-build")))]
                Some(TokenPricingSource::Mock { .. }) => {
                    err!(ErrorCode::FundOperationCommandExecutionFailedException)?
                }
            }))
        })()?
        else {
            // fallback: next vault
            return self.execute_new(ctx, Some(vault), previous_execution_result);
        };

        Ok((previous_execution_result, Some(entry)))
    }

    #[inline(never)]
    fn execute_prepare<'info>(
        &self,
        ctx: &OperationCommandContext,
        accounts: &[&'info AccountInfo<'info>],
        vault: &Pubkey,
    ) -> ExecutionResult {
        let fund_account = ctx.fund_account.load()?;
        let restaking_vault = fund_account.get_restaking_vault(vault)?;

        let Some(entry) = (|| match restaking_vault
            .receipt_token_pricing_source
            .try_deserialize()?
        {
            Some(TokenPricingSource::JitoRestakingVault { address }) => {
                let [vault_program, vault_config, vault_account, ..] = accounts else {
                    err!(error::ErrorCode::AccountNotEnoughKeys)?
                };
                require_keys_eq!(address, vault_account.key());

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
                            .flat_map(|index| {
                                let ticket_account = vault_service
                                    .find_withdrawal_ticket_account(&FundAccount::find_unrestaking_ticket_account_address(
                                        &ctx.fund_account.key(),
                                        vault,
                                        index,
                                    ));
                                let ticket_receipt_token_account =
                                    anchor_spl::associated_token::get_associated_token_address_with_program_id(
                                        &ticket_account,
                                        &restaking_vault.receipt_token_mint,
                                        &anchor_spl::token::ID,
                                    );
                                [(ticket_account, true), (ticket_receipt_token_account, true)]
                            }),
                    );
                let command = ClaimUnrestakedVSTCommand {
                    state: ClaimUnrestakedVSTCommandState::Execute { vault: *vault },
                };

                Ok(Some(command.with_required_accounts(required_accounts)))
            }
            // no unrestaking on virtual vault
            Some(TokenPricingSource::VirtualVault { .. }) => Ok(None),
            Some(TokenPricingSource::SolvBTCVault { address }) => {
                let [vault_program, vault_account, ..] = accounts else {
                    err!(error::ErrorCode::AccountNotEnoughKeys)?
                };
                require_keys_eq!(vault_account.key(), address);

                let vault_service = SolvBTCVaultService::new(vault_program, vault_account)?;
                let required_accounts = vault_service
                    .find_accounts_to_withdraw()?
                    .chain([
                        (
                            fund_account
                                .find_vault_supported_token_reserve_account_address(&address)?,
                            true,
                        ),
                        (
                            fund_account.find_vault_receipt_token_reserve_account_address(
                                &restaking_vault.vault,
                            )?,
                            true,
                        ),
                        (fund_account.get_reserve_account_address()?, false),
                    ])
                    .chain([
                        (fund_account.get_treasury_account_address()?, false),
                        (
                            fund_account.find_supported_token_treasury_account_address(
                                &vault_service.get_supported_token_mint()?,
                            )?,
                            true,
                        ),
                    ]);
                let command = ClaimUnrestakedVSTCommand {
                    state: ClaimUnrestakedVSTCommandState::Execute { vault: *vault },
                };

                Ok(Some(command.with_required_accounts(required_accounts)))
            }
            // invalid configuration
            Some(TokenPricingSource::SPLStakePool { .. })
            | Some(TokenPricingSource::MarinadeStakePool { .. })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::PeggedToken { .. })
            | None => err!(ErrorCode::FundOperationCommandExecutionFailedException),
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(ErrorCode::FundOperationCommandExecutionFailedException)
            }
        })()?
        else {
            // fallback: next vault
            return self.execute_new(ctx, Some(vault), None);
        };

        Ok((None, Some(entry)))
    }

    fn execute_execute<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        vault: &Pubkey,
    ) -> ExecutionResult {
        let fund_account = ctx.fund_account.load()?;
        let restaking_vault = fund_account.get_restaking_vault(vault)?;
        let supported_token_mint = restaking_vault.supported_token_mint;
        let receipt_token_pricing_source = restaking_vault
            .receipt_token_pricing_source
            .try_deserialize()?;

        drop(fund_account);

        let result = (|| match receipt_token_pricing_source {
            Some(TokenPricingSource::JitoRestakingVault { address }) => {
                let [vault_program, vault_config, vault_account, token_program, system_program, vault_receipt_token_mint, vault_program_fee_receipt_token_account, vault_fee_receipt_token_account, vault_supported_token_reserve_account, fund_vault_supported_token_reserve_account, fund_reserve_account, remaining_accounts @ ..] =
                    accounts
                else {
                    err!(error::ErrorCode::AccountNotEnoughKeys)?
                };
                require_keys_eq!(address, vault_account.key());

                let withdrawal_ticket_candidate_accounts = {
                    if remaining_accounts.len() < 10 {
                        err!(error::ErrorCode::AccountNotEnoughKeys)?
                    }
                    &remaining_accounts[..10]
                };

                let vault_service =
                    JitoRestakingVaultService::new(vault_program, vault_config, vault_account)?;

                let mut claimable_withdrawal_ticket_indices = vec![];
                for i in 0..5 {
                    let ticket = withdrawal_ticket_candidate_accounts[i * 2];
                    if vault_service.is_claimable_withdrawal_ticket(ticket)? {
                        claimable_withdrawal_ticket_indices.push(i);
                    }
                }

                let mut result_claimed_supported_token_amount = 0u64;
                let mut result_unrestaked_receipt_token_amount = 0u64;
                let mut result_deducted_receipt_token_fee_amount = 0u64;

                Ok(if !claimable_withdrawal_ticket_indices.is_empty() {
                    let mut pricing_service =
                        FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                            .new_pricing_service(accounts.iter().copied(), false)?;

                    let mut fund_account = ctx.fund_account.load_mut()?;
                    let restaking_vault = fund_account.get_restaking_vault_mut(vault)?;
                    let receipt_token_mint = &restaking_vault.receipt_token_mint;

                    let (supported_token_amount_numerator, receipt_token_amount_denominator) =
                        pricing_service.get_vault_supported_token_to_receipt_token_exchange_ratio(
                            receipt_token_mint,
                        )?;
                    restaking_vault.update_supported_token_to_receipt_token_exchange_ratio(
                        supported_token_amount_numerator,
                        receipt_token_amount_denominator,
                    )?;

                    drop(fund_account);

                    let mut last_to_vault_supported_token_amount = 0;

                    let fund_account = ctx.fund_account.load()?;
                    for i in claimable_withdrawal_ticket_indices {
                        let withdrawal_ticket_account = withdrawal_ticket_candidate_accounts[i * 2];
                        let withdrawal_ticket_receipt_token_account =
                            withdrawal_ticket_candidate_accounts[i * 2 + 1];

                        let (
                            to_vault_supported_token_amount,
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
                            &[&fund_account.get_reserve_account_seeds()],
                            ctx.operator,
                        )?;

                        require_gte!(
                            fund_reserve_account.lamports(),
                            fund_account.sol.get_total_reserved_amount()
                        );
                        result_claimed_supported_token_amount += claimed_supported_token_amount;
                        result_unrestaked_receipt_token_amount += unrestaked_receipt_token_amount;
                        result_deducted_receipt_token_fee_amount +=
                            deducted_program_fee_receipt_token_amount
                                + deducted_vault_fee_receipt_token_amount;

                        last_to_vault_supported_token_amount = to_vault_supported_token_amount;
                    }

                    drop(fund_account);

                    FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                        .update_asset_values(&mut pricing_service, false)?;

                    let mut fund_account = ctx.fund_account.load_mut()?;

                    let restaking_vault = fund_account.get_restaking_vault_mut(vault)?;
                    restaking_vault.receipt_token_operation_receivable_amount -=
                        result_unrestaked_receipt_token_amount;

                    let result_total_unrestaking_receipt_token_amount =
                        restaking_vault.receipt_token_operation_receivable_amount;

                    let result_operation_reserved_supported_token_amount: u64;
                    let result_operation_receivable_supported_token_amount: u64;
                    #[allow(clippy::wildcard_enum_match_arm)]
                    match fund_account.get_normalized_token_mut() {
                        Some(normalized_token) if normalized_token.mint == supported_token_mint => {
                            normalized_token.operation_reserved_amount +=
                                result_claimed_supported_token_amount;
                            result_operation_reserved_supported_token_amount =
                                normalized_token.operation_reserved_amount;

                            require_gte!(
                                last_to_vault_supported_token_amount,
                                normalized_token.operation_reserved_amount
                            );

                            fund_account.sol.operation_receivable_amount += pricing_service
                                .get_token_amount_as_sol(
                                    vault_receipt_token_mint.key,
                                    result_deducted_receipt_token_fee_amount,
                                )?;
                            result_operation_receivable_supported_token_amount = pricing_service
                                .get_sol_amount_as_token(
                                    &supported_token_mint,
                                    fund_account.sol.operation_receivable_amount,
                                )?;
                        }
                        _ => {
                            let supported_token =
                                fund_account.get_supported_token_mut(&supported_token_mint)?;
                            supported_token.token.operation_receivable_amount += pricing_service
                                .get_token_amount_as_token(
                                    vault_receipt_token_mint.key,
                                    result_deducted_receipt_token_fee_amount,
                                    &supported_token_mint,
                                )?;
                            supported_token.token.operation_reserved_amount +=
                                result_claimed_supported_token_amount;
                            result_operation_reserved_supported_token_amount =
                                supported_token.token.operation_reserved_amount;
                            result_operation_receivable_supported_token_amount =
                                supported_token.token.operation_receivable_amount;

                            require_gte!(
                                last_to_vault_supported_token_amount,
                                supported_token.token.operation_reserved_amount
                            );
                        }
                    };
                    drop(fund_account);

                    FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                        .update_asset_values(&mut pricing_service, true)?;

                    Some(
                        ClaimUnrestakedVSTCommandResult {
                            vault: *vault,
                            receipt_token_mint: *vault_receipt_token_mint.key,
                            total_unrestaking_receipt_token_amount:
                                result_total_unrestaking_receipt_token_amount,
                            unrestaked_receipt_token_amount: result_unrestaked_receipt_token_amount,
                            deducted_receipt_token_fee_amount:
                                result_deducted_receipt_token_fee_amount,

                            supported_token_mint,
                            claimed_supported_token_amount: result_claimed_supported_token_amount,
                            transferred_supported_token_revenue_amount: 0,
                            offsetted_supported_token_receivable_amount: 0,
                            offsetted_asset_receivables: vec![],
                            operation_reserved_supported_token_amount:
                                result_operation_reserved_supported_token_amount,
                            operation_receivable_supported_token_amount:
                                result_operation_receivable_supported_token_amount,
                        }
                        .into(),
                    )
                } else {
                    None
                })
            }
            // no unrestaking on virtual vault
            Some(TokenPricingSource::VirtualVault { .. }) => Ok(None),
            Some(TokenPricingSource::SolvBTCVault { address }) => {
                let [vault_program, vault_account, vault_receipt_token_mint, vault_supported_token_mint, vault_vault_supported_token_account, token_program, event_authority, fund_reserve_vault_supported_token_account, fund_reserve_vault_receipt_token_account, fund_reserve, fund_treasury, fund_treasury_vault_supported_token_account, ..] =
                    accounts
                else {
                    err!(error::ErrorCode::AccountNotEnoughKeys)?
                };
                require_keys_eq!(address, vault_account.key());

                let mut pricing_service =
                    FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                        .new_pricing_service(accounts.iter().copied(), false)?;

                let mut fund_account = ctx.fund_account.load_mut()?;
                let restaking_vault: &mut RestakingVault =
                    fund_account.get_restaking_vault_mut(vault)?;
                let receipt_token_mint: &Pubkey = &restaking_vault.receipt_token_mint;

                let (supported_token_amount_numerator, receipt_token_amount_denominator) =
                    pricing_service.get_vault_supported_token_to_receipt_token_exchange_ratio(
                        receipt_token_mint,
                    )?;
                restaking_vault.update_supported_token_to_receipt_token_exchange_ratio(
                    supported_token_amount_numerator,
                    receipt_token_amount_denominator,
                )?;

                drop(fund_account);

                let fund_account = ctx.fund_account.load()?;

                let vault_service = SolvBTCVaultService::new(vault_program, vault_account)?;
                let (
                    fund_reserve_vault_supported_token_amount,
                    unrestaked_receipt_token_amount,
                    claimed_supported_token_amount,
                    deducted_supported_token_fee_amount,
                    total_unrestaking_receipt_token_amount,
                ) = vault_service.withdraw(
                    vault_receipt_token_mint,
                    vault_supported_token_mint,
                    vault_vault_supported_token_account,
                    token_program,
                    event_authority,
                    ctx.fund_account.as_ref(),
                    &[&fund_account.get_seeds()],
                    fund_reserve_vault_receipt_token_account,
                    fund_reserve_vault_supported_token_account,
                    fund_reserve,
                    &[&fund_account.get_reserve_account_seeds()],
                )?;

                if unrestaked_receipt_token_amount == 0 {
                    return Ok(None);
                }

                drop(fund_account);

                let mut fund_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?;
                fund_service.update_asset_values(&mut pricing_service, false)?;

                let deducted_receipt_token_fee_amount = pricing_service.get_token_amount_as_token(
                    &supported_token_mint,
                    deducted_supported_token_fee_amount,
                    vault_receipt_token_mint.key,
                )?;

                let (
                    transferred_supported_token_revenue_amount,
                    offsetted_supported_token_receivable_amount,
                    offsetted_asset_receivables,
                ) = fund_service.offset_receivables(
                    ctx.system_program,
                    fund_reserve,
                    fund_treasury,
                    Some(vault_supported_token_mint),
                    Some(token_program),
                    Some(fund_reserve_vault_supported_token_account),
                    Some(fund_treasury_vault_supported_token_account),
                    claimed_supported_token_amount,
                    &pricing_service,
                )?;

                fund_service.update_asset_values(&mut pricing_service, true)?;

                drop(fund_service);

                let fund = ctx.fund_account.load()?;
                let supported_token = fund.get_supported_token(&supported_token_mint)?;

                require_gte!(
                    fund_reserve_vault_supported_token_amount,
                    supported_token.token.operation_reserved_amount
                );

                Ok(Some(
                    ClaimUnrestakedVSTCommandResult {
                        vault: *vault,
                        receipt_token_mint: *vault_receipt_token_mint.key,
                        total_unrestaking_receipt_token_amount,
                        unrestaked_receipt_token_amount,
                        deducted_receipt_token_fee_amount,
                        supported_token_mint,
                        claimed_supported_token_amount,
                        transferred_supported_token_revenue_amount,
                        offsetted_supported_token_receivable_amount,
                        offsetted_asset_receivables: offsetted_asset_receivables
                            .into_iter()
                            .map(|(asset_token_mint, asset_amount)| {
                                ClaimUnrestakedVSTCommandResultAssetReceivable {
                                    asset_token_mint,
                                    asset_amount,
                                }
                            })
                            .collect::<Vec<_>>(),
                        operation_reserved_supported_token_amount: supported_token
                            .token
                            .operation_reserved_amount,
                        operation_receivable_supported_token_amount: supported_token
                            .token
                            .operation_receivable_amount,
                    }
                    .into(),
                ))
            }
            // invalid configuration
            Some(TokenPricingSource::SPLStakePool { .. })
            | Some(TokenPricingSource::MarinadeStakePool { .. })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::PeggedToken { .. })
            | None => err!(ErrorCode::FundOperationCommandExecutionFailedException),
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(ErrorCode::FundOperationCommandExecutionFailedException)
            }
        })()?;

        // Move on to next vault
        self.execute_new(ctx, Some(vault), result)
    }
}
