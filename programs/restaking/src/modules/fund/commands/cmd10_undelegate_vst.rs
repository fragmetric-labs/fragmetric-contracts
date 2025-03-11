use anchor_lang::prelude::*;

use crate::{
    errors,
    modules::{pricing::TokenPricingSource, restaking::JitoRestakingVaultService},
    utils::PDASeeds,
};

use super::{
    OperationCommandContext, OperationCommandEntry, OperationCommandResult, SelfExecutable,
    StakeSOLCommand, FUND_ACCOUNT_MAX_RESTAKING_VAULTS,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct UndelegateVSTCommand {
    state: UndelegateVSTCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct UndelegateVSTCommandItem {
    vault: Pubkey,
    operator: Pubkey,
    undelegation_amount: u64,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum UndelegateVSTCommandState {
    #[default]
    New,
    Prepare {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        items: Vec<UndelegateVSTCommandItem>,
    },
    Execute {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        items: Vec<UndelegateVSTCommandItem>,
    },
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct UndelegateVSTCommandResult {
    pub vault_supported_token_mint: Pubkey,
    pub requested_undelegation_token_amount: u64,
    pub total_delegated_token_amount: u64,
    pub total_undelegating_token_amount: u64,
}

impl SelfExecutable for UndelegateVSTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let (result, entry) = match &self.state {
            UndelegateVSTCommandState::New => self.execute_new(ctx, accounts)?,
            UndelegateVSTCommandState::Prepare { items } => {
                self.execute_prepare(ctx, accounts, items.clone(), None)?
            }
            UndelegateVSTCommandState::Execute { items } => {
                self.execute_execute(ctx, accounts, items)?
            }
        };

        Ok((
            result,
            entry.or_else(|| Some(StakeSOLCommand::default().without_required_accounts())),
        ))
    }
}

impl UndelegateVSTCommand {
    fn execute_new<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        _accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let mut items =
            Vec::<UndelegateVSTCommandItem>::with_capacity(FUND_ACCOUNT_MAX_RESTAKING_VAULTS);
        // FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS

        ctx.fund_account
            .load()?
            .get_restaking_vaults_iter()
            .for_each(|restaking_vault| {
                restaking_vault
                    .get_delegations_iter()
                    .for_each(|delegation| {
                        items.push(UndelegateVSTCommandItem {
                            vault: restaking_vault.vault,
                            operator: delegation.operator,
                            undelegation_amount: delegation.supported_token_delegated_amount,
                        });
                    });
            });

        // nothing to undelegate
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
            state: UndelegateVSTCommandState::Prepare { items },
        }
        .with_required_accounts(required_accounts);

        Ok((None, Some(command)))
    }

    fn execute_prepare<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        items: Vec<UndelegateVSTCommandItem>,
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
                        .find_accounts_to_cooldown_delegation(item.operator)?;

                let command = Self {
                    state: UndelegateVSTCommandState::Execute { items },
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
        items: &[UndelegateVSTCommandItem],
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

                vault_service.cooldown_delegation(
                    vault_operator,
                    vault_operator_delegation,
                    vault_delegation_admin,
                    fund_account.get_seeds().as_ref(),
                    item.undelegation_amount,
                )?;

                drop(fund_account);
                let mut fund_account = ctx.fund_account.load_mut()?;

                {
                    let restaking_vault = fund_account.get_restaking_vault_mut(&item.vault)?;
                    restaking_vault.receipt_token_operation_reserved_amount +=
                        item.undelegation_amount;

                    let delegation = restaking_vault.get_delegation_mut(&item.operator)?;
                    delegation.supported_token_delegated_amount -= item.undelegation_amount;
                    delegation.supported_token_undelegating_amount += item.undelegation_amount;
                }

                let restaking_vault = fund_account.get_restaking_vault(&item.vault)?;
                let delegation = restaking_vault.get_delegation(&item.operator)?;

                let result = Some(
                    UndelegateVSTCommandResult {
                        vault_supported_token_mint: restaking_vault.supported_token_mint,
                        requested_undelegation_token_amount: item.undelegation_amount,
                        total_delegated_token_amount: delegation
                            .supported_token_delegated_amount,
                        total_undelegating_token_amount: delegation
                            .supported_token_undelegating_amount,
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
