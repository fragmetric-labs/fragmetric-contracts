use anchor_lang::prelude::*;

use crate::modules::fund::{
    WeightedAllocationParticipant, FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS,
};
use crate::modules::pricing::TokenPricingSource;
use crate::modules::restaking::JitoRestakingVaultService;
use crate::utils::PDASeeds;
use crate::{errors, modules::fund::WeightedAllocationStrategy};

use super::{
    OperationCommandContext, OperationCommandEntry, OperationCommandResult, SelfExecutable,
    FUND_ACCOUNT_MAX_RESTAKING_VAULTS,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct DelegateVSTCommand {
    state: DelegateVSTCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct DelegateVSTCommandItem {
    operator: Pubkey,
    delegation_amount: u64,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum DelegateVSTCommandState {
    #[default]
    New,
    Prepare {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        vaults: Vec<Pubkey>,

        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS)]
        items: Vec<DelegateVSTCommandItem>,
    },
    Execute {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        vaults: Vec<Pubkey>,

        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS)]
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
            DelegateVSTCommandState::Prepare { vaults, items } => {
                self.execute_prepare(ctx, accounts, vaults.clone(), items.clone(), None)?
            }
            DelegateVSTCommandState::Execute { vaults, items } => {
                self.execute_execute(ctx, accounts, vaults, items)?
            }
        };

        Ok((result, entry))
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
        let fund_account = ctx.fund_account.load()?;
        let vaults = fund_account
            .get_restaking_vaults_iter()
            .map(|vault| vault.vault);

        Ok((None, self.create_prepare_command(ctx, vaults)?))
    }

    fn create_prepare_command<'info>(
        &self,
        ctx: &OperationCommandContext<'info, '_>,
        vaults: impl Iterator<Item = Pubkey>,
    ) -> Result<Option<OperationCommandEntry>> {
        let fund_account = ctx.fund_account.load()?;
        let vaults = vaults.collect::<Vec<Pubkey>>();

        let vault = &vaults[0];
        let mut items = Vec::<DelegateVSTCommandItem>::with_capacity(
            FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS,
        );

        let restaking_vault = fund_account.get_restaking_vault(vault)?;

        let mut strategy =
            WeightedAllocationStrategy::<FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS>::new(
                restaking_vault.get_delegations_iter().map(|delegation| {
                    items.push(DelegateVSTCommandItem {
                        operator: delegation.operator,
                        delegation_amount: 0,
                    });

                    WeightedAllocationParticipant::new(
                        delegation.supported_token_allocation_weight,
                        delegation.supported_token_delegated_amount,
                        delegation.supported_token_allocation_capacity_amount,
                    )
                }),
            );
        strategy.put(restaking_vault.receipt_token_operation_reserved_amount)?;

        const MIN_ALLOCATED_TOKEN_AMOUNT: u64 = 1_000_000_000;
        for (index, _participant) in strategy.get_participants_iter().enumerate() {
            let allocated_token_amount =
                strategy.get_participant_last_put_amount_by_index(index)?;

            if allocated_token_amount >= MIN_ALLOCATED_TOKEN_AMOUNT {
                if let Some(item) = items.get_mut(index) {
                    item.delegation_amount = allocated_token_amount;
                };
            }
        }
        items.retain(|item| item.delegation_amount >= MIN_ALLOCATED_TOKEN_AMOUNT);

        if vaults.is_empty() || items.is_empty() {
            return Ok(None);
        }

        let vault = &vaults[0];
        let required_accounts = JitoRestakingVaultService::find_accounts_to_new(*vault)?;

        let command = Self {
            state: DelegateVSTCommandState::Prepare { vaults, items },
        }
        .with_required_accounts(required_accounts);

        Ok(Some(command))
    }

    fn execute_prepare<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        vaults: Vec<Pubkey>,
        items: Vec<DelegateVSTCommandItem>,
        previous_execution_result: Option<OperationCommandResult>,
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if vaults.is_empty() {
            return Ok((previous_execution_result, None));
        }
        if items.is_empty() {
            let vaults = vaults[1..].iter().copied();
            return Ok((
                previous_execution_result,
                self.create_prepare_command(ctx, vaults)?,
            ));
        }

        let vault = &vaults[0];
        let item = &items[0];

        let fund_account = ctx.fund_account.load()?;
        let restaking_vault = fund_account.get_restaking_vault(vault)?;

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
                    state: DelegateVSTCommandState::Execute { vaults, items },
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
        vaults: &[Pubkey],
        items: &[DelegateVSTCommandItem],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if vaults.is_empty() {
            return Ok((None, None));
        }
        if items.is_empty() {
            let vaults = vaults[1..].iter().copied();
            return Ok((None, self.create_prepare_command(ctx, vaults)?));
        }
        let vault = &vaults[0];
        let item = &items[0];

        let fund_account = ctx.fund_account.load()?;
        let restaking_vault = fund_account.get_restaking_vault(vault)?;

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
                    let restaking_vault = fund_account.get_restaking_vault_mut(vault)?;
                    restaking_vault.receipt_token_operation_reserved_amount -=
                        item.delegation_amount;

                    let delegation = restaking_vault.get_delegation_mut(&item.operator)?;
                    delegation.supported_token_delegated_amount += item.delegation_amount;
                }

                let restaking_vault = fund_account.get_restaking_vault(vault)?;
                let delegation = restaking_vault.get_delegation(&item.operator)?;

                let result = Some(
                    DelegateVSTCommandResult {
                        vault_supported_token_mint: restaking_vault.supported_token_mint,
                        delegated_token_amount: item.delegation_amount,
                        total_delegated_token_amount: delegation.supported_token_delegated_amount,
                    }
                    .into(),
                );

                drop(fund_account);

                // prepare state does not require additional accounts,
                // so we can execute directly.
                let items = &items[1..];
                if items.is_empty() {
                    let entry = self.create_prepare_command(ctx, vaults[1..].iter().copied())?;
                    Ok((result, entry))
                } else {
                    self.execute_prepare(ctx, accounts, vaults.to_vec(), items.to_vec(), result)
                }
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
