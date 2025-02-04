use anchor_lang::prelude::*;

use crate::errors;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::restaking::JitoRestakingVaultService;
use crate::utils::PDASeeds;

use super::{
    HarvestRewardCommand, OperationCommandContext, OperationCommandEntry, OperationCommandResult,
    SelfExecutable, FUND_ACCOUNT_MAX_RESTAKING_VAULTS,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct DelegateVSTCommand {
    state: DelegateVSTCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct DelegateVSTCommandItem {
    vault: Pubkey,
    operator: Pubkey,
    delegation_amount: u64,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum DelegateVSTCommandState {
    #[default]
    New,
    Prepare {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        items: Vec<DelegateVSTCommandItem>,
    },
    Execute {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        items: Vec<DelegateVSTCommandItem>,
    },
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct DelegateVSTCommandResult {
    pub vault_supported_token_mint: Pubkey,
    pub delegated_token_amount: u64,
    pub total_delegated_token_amount: u64,
}

impl SelfExecutable for DelegateVSTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let (result, entry) = match &self.state {
            DelegateVSTCommandState::New => self.execute_new(ctx, accounts)?,
            DelegateVSTCommandState::Prepare { items } => {
                self.execute_prepare(ctx, accounts, items.clone(), None)?
            }
            DelegateVSTCommandState::Execute { items } => {
                self.execute_execute(ctx, accounts, items)?
            }
        };

        Ok((
            result,
            entry.or_else(|| Some(HarvestRewardCommand::default().without_required_accounts())),
        ))
    }
}

#[deny(clippy::wildcard_enum_match_arm)]
impl DelegateVSTCommand {
    fn execute_new<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        _accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        // TODO v0.4.3: items size capacity should be changed for more acurate cf. unstake command
        let mut items =
            Vec::<DelegateVSTCommandItem>::with_capacity(FUND_ACCOUNT_MAX_RESTAKING_VAULTS);

        ctx.fund_account
            .load()?
            .get_restaking_vaults_iter()
            .for_each(|restaking_vault| {
                let num_operators = restaking_vault.get_delegations_iter().count();
                restaking_vault
                    .get_delegations_iter()
                    .for_each(|delegation| {
                        items.push(DelegateVSTCommandItem {
                            vault: restaking_vault.vault,
                            operator: delegation.operator,
                            delegation_amount: restaking_vault
                                .receipt_token_operation_reserved_amount
                                .saturating_div(num_operators as u64),
                        });
                    });
            });

        // nothing to delegate
        if items.is_empty() {
            return Ok((None, None));
        }

        let pricing_source = ctx
            .fund_account
            .load()?
            .get_restaking_vault(&items.first().unwrap().vault)?
            .receipt_token_pricing_source
            .try_deserialize()?;

        let required_accounts = match pricing_source {
            Some(TokenPricingSource::JitoRestakingVault { address }) => {
                JitoRestakingVaultService::find_accounts_to_new(address)?
            }
            // otherwise fails
            Some(TokenPricingSource::SPLStakePool { .. })
            | Some(TokenPricingSource::MarinadeStakePool { .. })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | None => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        };

        let command = Self {
            state: DelegateVSTCommandState::Prepare { items },
        }
        .with_required_accounts(required_accounts);

        Ok((None, Some(command)))
    }

    fn execute_prepare<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        items: Vec<DelegateVSTCommandItem>,
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
        let restaking_vault = fund_account.get_restaking_vault(&item.vault)?;

        match restaking_vault
            .receipt_token_pricing_source
            .try_deserialize()?
        {
            Some(TokenPricingSource::JitoRestakingVault { address }) => {
                let [vault_program, vault_config, vault_account, _remaining_accounts @ ..] =
                    accounts
                else {
                    err!(ErrorCode::AccountNotEnoughKeys)?
                };
                require_keys_eq!(address, vault_account.key());

                let required_accounts =
                    JitoRestakingVaultService::new(vault_program, vault_config, vault_account)?
                        .find_accounts_to_add_delegation(item.operator)?;

                let command = Self {
                    state: DelegateVSTCommandState::Execute { items },
                }
                .with_required_accounts(required_accounts);

                Ok((previous_execution_result, Some(command)))
            }
            // otherwise fails
            Some(TokenPricingSource::SPLStakePool { .. })
            | Some(TokenPricingSource::MarinadeStakePool { .. })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
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

    fn execute_execute<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        items: &[DelegateVSTCommandItem],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if items.is_empty() {
            return Ok((None, None));
        }
        let item = items[0];

        let fund_account = ctx.fund_account.load()?;
        let restaking_vault = fund_account.get_restaking_vault(&item.vault)?;

        match restaking_vault
            .receipt_token_pricing_source
            .try_deserialize()?
        {
            Some(TokenPricingSource::JitoRestakingVault { address }) => {
                let [vault_program, vault_config, vault_account, vault_operator, vault_operator_delegation, vault_delegation_admin, ..] =
                    accounts
                else {
                    err!(ErrorCode::AccountNotEnoughKeys)?
                };
                require_keys_eq!(address, vault_account.key());
                require_keys_eq!(vault_delegation_admin.key(), ctx.fund_account.key());

                let vault_service =
                    JitoRestakingVaultService::new(vault_program, vault_config, vault_account)?;

                vault_service.add_delegation(
                    vault_operator,
                    vault_operator_delegation,
                    vault_delegation_admin,
                    fund_account.get_seeds().as_ref(),
                    item.delegation_amount,
                )?;

                drop(fund_account);
                let mut fund_account = ctx.fund_account.load_mut()?;

                {
                    let restaking_vault = fund_account.get_restaking_vault_mut(&item.vault)?;
                    restaking_vault.receipt_token_operation_reserved_amount -=
                        item.delegation_amount;

                    let delegation = restaking_vault.get_delegation_mut(&item.operator)?;
                    delegation.supported_token_delegated_amount += item.delegation_amount;
                }

                let restaking_vault = fund_account.get_restaking_vault(&item.vault)?;
                let delegation = restaking_vault.get_delegation(&item.operator)?;

                let result = Some(
                    DelegateVSTCommandResult {
                        vault_supported_token_mint: restaking_vault.supported_token_mint,
                        delegated_token_amount: item.delegation_amount,
                        total_delegated_token_amount: delegation.supported_token_delegated_amount,
                    }
                    .into(),
                );

                if items.is_empty() {
                    return Ok((result, None));
                }

                // prepare state does not require additional accounts,
                // so we can execute directly.
                drop(fund_account);
                self.execute_prepare(ctx, accounts, items[1..].to_vec(), result)
            }
            // otherwise fails
            Some(TokenPricingSource::SPLStakePool { .. })
            | Some(TokenPricingSource::MarinadeStakePool { .. })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
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
}
