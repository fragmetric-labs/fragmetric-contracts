use anchor_lang::prelude::*;

use crate::{
    errors,
    modules::{
        fund::{
            FundService, WeightedAllocationParticipant, WeightedAllocationStrategy,
            FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS,
        },
        pricing::TokenPricingSource,
        restaking::JitoRestakingVaultService,
    },
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
    operator: Pubkey,
    undelegation_amount: u64,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum UndelegateVSTCommandState {
    #[default]
    New,
    PrepareItems {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        vaults: Vec<Pubkey>,
    },
    Prepare {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        vaults: Vec<Pubkey>,

        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS)]
        items: Vec<UndelegateVSTCommandItem>,
    },
    Execute {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        vaults: Vec<Pubkey>,

        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS)]
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
            UndelegateVSTCommandState::PrepareItems { vaults } => {
                self.execute_prepare_items(ctx, accounts, vaults.clone())?
            }
            UndelegateVSTCommandState::Prepare { vaults, items } => {
                self.execute_prepare(ctx, accounts, vaults.clone(), items.clone(), None)?
            }
            UndelegateVSTCommandState::Execute { vaults, items } => {
                self.execute_execute(ctx, accounts, vaults, items)?
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
        let fund_account = ctx.fund_account.load()?;
        let vaults = fund_account
            .get_restaking_vaults_iter()
            .map(|vault| vault.vault)
            .collect::<Vec<Pubkey>>();

        let required_accounts = JitoRestakingVaultService::find_accounts_to_new(vaults[0])?;

        let command = Self {
            state: UndelegateVSTCommandState::PrepareItems { vaults },
        }
        .with_required_accounts(required_accounts);

        Ok((None, Some(command)))
    }

    fn execute_prepare_items<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        vaults: Vec<Pubkey>,
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let pricing_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
            .new_pricing_service(accounts.iter().copied())?;
        let fund_account = ctx.fund_account.load()?;
        let mut items = Vec::<UndelegateVSTCommandItem>::with_capacity(
            FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS,
        );

        let restaking_vault = fund_account.get_restaking_vault(&vaults[0])?;

        let mut strategy =
            WeightedAllocationStrategy::<FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS>::new(
                restaking_vault.get_delegations_iter().map(|delegation| {
                    items.push(UndelegateVSTCommandItem {
                        operator: delegation.operator,
                        undelegation_amount: 0,
                    });

                    WeightedAllocationParticipant::new(
                        delegation.supported_token_allocation_weight,
                        delegation.supported_token_delegated_amount,
                        delegation.supported_token_allocation_capacity_amount,
                    )
                }),
            );

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

                let vault_service =
                    JitoRestakingVaultService::new(vault_program, vault_config, vault_account)?;

                let vault_requested_unrestake_amount_as_vst_amount = pricing_service
                    .get_token_amount_as_asset(
                        &restaking_vault.receipt_token_mint,
                        vault_service.get_vault_requested_unrestake_amount()?,
                        Some(&restaking_vault.supported_token_mint),
                    )?;
                let vault_requested_undelegate_amount =
                    vault_service.get_vault_requested_undelegate_amount()?;
                let vault_supported_token_remaining_amount =
                    vault_service.get_vault_supported_token_remaining_amount()?;
                msg!(
                    "vault_requested_unrestake_amount_as_vst_amount {}, vault_requested_undelegate_amount {}, vault_supported_token_remaining_amount {}",
                    vault_requested_unrestake_amount_as_vst_amount,
                    vault_requested_undelegate_amount,
                    vault_supported_token_remaining_amount
                );
                let undelegation_amount = vault_requested_unrestake_amount_as_vst_amount
                    .saturating_sub(vault_requested_undelegate_amount)
                    .saturating_sub(vault_supported_token_remaining_amount);
                msg!("willing to undelegation_amount {}", undelegation_amount);

                strategy.cut_greedy(undelegation_amount)?;

                const MIN_ALLOCATED_TOKEN_AMOUNT: u64 = 1_000_000_000;
                for (index, _participant) in strategy.get_participants_iter().enumerate() {
                    let allocated_token_amount =
                        strategy.get_participant_last_cut_amount_by_index(index)?;
                    msg!(
                        "index {}, item.operator {}, allocated_token_amount {}",
                        index,
                        items.get(index).unwrap().operator,
                        allocated_token_amount
                    );

                    if allocated_token_amount >= MIN_ALLOCATED_TOKEN_AMOUNT {
                        if let Some(item) = items.get_mut(index) {
                            item.undelegation_amount = allocated_token_amount;
                        }
                    }
                }
                items.retain(|item| item.undelegation_amount >= MIN_ALLOCATED_TOKEN_AMOUNT);

                if vaults.is_empty() && items.is_empty() {
                    return Ok((None, None));
                }

                let required_accounts = JitoRestakingVaultService::find_accounts_to_new(vaults[0])?;
                let command = Self {
                    state: UndelegateVSTCommandState::Prepare { vaults, items },
                }
                .with_required_accounts(required_accounts);

                Ok((None, Some(command)))
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

    fn execute_prepare<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        vaults: Vec<Pubkey>,
        items: Vec<UndelegateVSTCommandItem>,
        previous_execution_result: Option<OperationCommandResult>,
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if vaults.is_empty() {
            return Ok((previous_execution_result, None));
        }
        if items.is_empty() {
            let vaults = vaults[1..].to_vec();

            if vaults.len() > 0 {
                let required_accounts = JitoRestakingVaultService::find_accounts_to_new(vaults[0])?;
                let command = Self {
                    state: UndelegateVSTCommandState::PrepareItems { vaults },
                }
                .with_required_accounts(required_accounts);

                return Ok((previous_execution_result, Some(command)));
            } else {
                return Ok((previous_execution_result, None));
            }
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
                        .find_accounts_to_cooldown_delegation(item.operator)?;

                let command = Self {
                    state: UndelegateVSTCommandState::Execute { vaults, items },
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
        items: &[UndelegateVSTCommandItem],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if vaults.is_empty() {
            return Ok((None, None));
        }
        if items.is_empty() {
            let vaults = vaults[1..].to_vec();
            return self.execute_prepare_items(ctx, accounts, vaults);
        }
        let vault = &vaults[0];
        let item = items[0];

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
                    let restaking_vault = fund_account.get_restaking_vault_mut(vault)?;
                    restaking_vault.receipt_token_operation_reserved_amount +=
                        item.undelegation_amount;

                    let delegation = restaking_vault.get_delegation_mut(&item.operator)?;
                    delegation.supported_token_delegated_amount -= item.undelegation_amount;
                    delegation.supported_token_undelegating_amount += item.undelegation_amount;
                }

                let restaking_vault = fund_account.get_restaking_vault(vault)?;
                let delegation = restaking_vault.get_delegation(&item.operator)?;

                let result = Some(
                    UndelegateVSTCommandResult {
                        vault_supported_token_mint: restaking_vault.supported_token_mint,
                        requested_undelegation_token_amount: item.undelegation_amount,
                        total_delegated_token_amount: delegation.supported_token_delegated_amount,
                        total_undelegating_token_amount: delegation
                            .supported_token_undelegating_amount,
                    }
                    .into(),
                );

                drop(fund_account);

                // prepare state does not require additional accounts,
                // so we can execute directly.
                let items = &items[1..];
                if items.is_empty() {
                    self.execute_prepare_items(ctx, accounts, vaults.to_vec())
                } else {
                    self.execute_prepare(
                        ctx,
                        accounts,
                        vaults.to_vec(),
                        items[1..].to_vec(),
                        result,
                    )
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
