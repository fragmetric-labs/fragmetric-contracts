use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::restaking::JitoRestakingVaultService;

use super::{
    EnqueueWithdrawalBatchCommand, OperationCommandContext, OperationCommandEntry,
    OperationCommandResult, SelfExecutable, FUND_ACCOUNT_MAX_RESTAKING_VAULTS,
    FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct InitializeCommand {
    state: InitializeCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Default)]
pub enum InitializeCommandState {
    /// Initializes a command with items based on the fund state and strategy.
    #[default]
    NewRestakingVaultUpdate,
    /// Prepares to execute restaking vault epoch process for the first item in the list.
    PrepareRestakingVaultUpdate {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        vaults: Vec<Pubkey>,
    },
    /// Executes restaking vault epoch process for the first item and transitions to the next command,
    /// either preparing the next item or performing a withdrawal operation.
    ExecuteRestakingVaultUpdate {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        vaults: Vec<Pubkey>,
        /// Items could be empty.
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS)]
        items: Vec<InitializeCommandRestakingVaultDelegationUpdateItem>,
    },
}

use InitializeCommandState::*;

impl std::fmt::Debug for InitializeCommandState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn debug_vault(
            f: &mut std::fmt::Formatter,
            variant: &'static str,
            vaults: &[Pubkey],
        ) -> std::fmt::Result {
            if vaults.is_empty() {
                return f.write_str(variant);
            }
            f.debug_struct(variant).field("vault", &vaults[0]).finish()
        }

        match self {
            Self::NewRestakingVaultUpdate => write!(f, "NewRestakingVaultUpdate"),
            Self::PrepareRestakingVaultUpdate { vaults, .. } => {
                debug_vault(f, "PrepareRestakingVaultUpdate", vaults)
            }
            Self::ExecuteRestakingVaultUpdate { vaults, .. } => {
                debug_vault(f, "ExecuteRestakingVaultUpdate", vaults)
            }
        }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize)]
pub struct InitializeCommandRestakingVaultDelegationUpdateItem {
    pub operator: Pubkey,
    pub index: u64,
}

const RESTAKING_VAULT_UPDATE_DELEGATIONS_BATCH_SIZE: usize = 10;

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct InitializeCommandResult {
    pub restaking_vault_updated: Option<InitializeCommandResultRestakingVaultUpdated>,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct InitializeCommandResultRestakingVaultUpdated {
    pub vault: Pubkey,
    pub epoch: u64,
    pub finalized: bool,
    pub supported_token_mint: Pubkey,
    pub delegations: Vec<InitializeCommandResultRestakingVaultDelegationUpdate>,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct InitializeCommandResultRestakingVaultDelegationUpdate {
    pub operator: Pubkey,
    pub delegated_amount: u64,
    pub undelegating_amount: u64,
}

impl SelfExecutable for InitializeCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let (result, entry) = match &self.state {
            NewRestakingVaultUpdate => self.execute_new_restaking_vault_update(ctx)?,
            PrepareRestakingVaultUpdate { vaults } => {
                self.execute_prepare_restaking_vault_update(ctx, accounts, vaults)?
            }
            ExecuteRestakingVaultUpdate { vaults, items } => {
                self.execute_execute_restaking_vault_update(ctx, accounts, vaults, items)?
            }
        };

        Ok((
            result,
            entry.or_else(|| {
                Some(EnqueueWithdrawalBatchCommand::default().without_required_accounts())
            }),
        ))
    }
}

#[deny(clippy::wildcard_enum_match_arm)]
impl InitializeCommand {
    #[inline(never)]
    fn execute_new_restaking_vault_update<'info>(
        &self,
        ctx: &OperationCommandContext<'info, '_>,
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        // 1. restaking_vault_update
        let fund_account = ctx.fund_account.load()?;
        let mut vaults = Vec::with_capacity(FUND_ACCOUNT_MAX_RESTAKING_VAULTS);
        for restaking_vault in fund_account.get_restaking_vaults_iter() {
            vaults.push(restaking_vault.vault);
        }

        if let Some(entry) = self.create_prepare_restaking_vault_update_command(ctx, vaults)? {
            return Ok((None, Some(entry)));
        }

        // // move on to next initialize operation
        // self.execute_new_another_operation(ctx, None)
        Ok((None, None))
    }

    fn create_prepare_restaking_vault_update_command<'info>(
        &self,
        ctx: &OperationCommandContext<'info, '_>,
        vaults: Vec<Pubkey>,
    ) -> Result<Option<OperationCommandEntry>> {
        if vaults.is_empty() {
            return Ok(None);
        }
        let receipt_token_pricing_source = ctx
            .fund_account
            .load()?
            .get_restaking_vault(&vaults[0])?
            .receipt_token_pricing_source
            .try_deserialize()?;

        let command = Self {
            state: PrepareRestakingVaultUpdate { vaults },
        };
        let entry = match receipt_token_pricing_source {
            Some(TokenPricingSource::JitoRestakingVault { address }) => {
                let required_accounts = JitoRestakingVaultService::find_accounts_to_new(address)?;
                command.with_required_accounts(required_accounts)
            }
            // otherwise fails
            Some(TokenPricingSource::SPLStakePool { .. })
            | Some(TokenPricingSource::MarinadeStakePool { .. })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        };

        Ok(Some(entry))
    }

    #[inline(never)]
    fn execute_prepare_restaking_vault_update<'info>(
        &self,
        ctx: &OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        vaults: &[Pubkey],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if vaults.is_empty() {
            // // move on to next initialize operation
            // return self.execute_new_another_operation(ctx, None);
            return Ok((None, None));
        }
        let fund_account = ctx.fund_account.load()?;
        let restaking_vault = fund_account.get_restaking_vault(&vaults[0])?;

        let receipt_token_pricing_source = restaking_vault
            .receipt_token_pricing_source
            .try_deserialize()?;
        match receipt_token_pricing_source {
            Some(TokenPricingSource::JitoRestakingVault { address }) => {
                let [vault_program, vault_config, vault_account, ..] = accounts else {
                    err!(error::ErrorCode::AccountNotEnoughKeys)?
                };
                require_keys_eq!(address, vault_account.key());

                let vault_service =
                    JitoRestakingVaultService::new(vault_program, vault_config, vault_account)?;

                // find items
                let ordered_indices = vault_service.get_ordered_vault_update_indices();

                let mut items = Vec::with_capacity(FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS);
                for index in ordered_indices {
                    let operator = restaking_vault
                        .get_delegations_iter()
                        .skip(index as usize)
                        .next()
                        .ok_or_else(|| {
                            error!(ErrorCode::FundOperationCommandExecutionFailedException)
                        })?
                        .operator;
                    items.push(InitializeCommandRestakingVaultDelegationUpdateItem {
                        operator,
                        index,
                    });
                }

                // create next command entry
                let operators = items
                    .iter()
                    .take(RESTAKING_VAULT_UPDATE_DELEGATIONS_BATCH_SIZE)
                    .map(|item| item.operator)
                    .collect::<Vec<_>>(); // this may be empty
                let accounts_to_update_vault_delegation_state =
                    vault_service.find_accounts_to_update_vault_delegation_state()?;
                let accounts_to_update_operator_delegation_state =
                    operators.iter().flat_map(|operator| {
                        vault_service.find_accounts_to_update_operator_delegation_state(*operator)
                    });
                let required_accounts = accounts_to_update_vault_delegation_state
                    .chain(accounts_to_update_operator_delegation_state);
                let command = Self {
                    state: ExecuteRestakingVaultUpdate {
                        vaults: vaults.to_vec(),
                        items,
                    },
                };

                Ok((
                    None,
                    Some(command.with_required_accounts(required_accounts)),
                ))
            }
            // otherwise fails
            Some(TokenPricingSource::SPLStakePool { .. })
            | Some(TokenPricingSource::MarinadeStakePool { .. })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        }
    }

    #[inline(never)]
    fn execute_execute_restaking_vault_update<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        vaults: &[Pubkey],
        items: &[InitializeCommandRestakingVaultDelegationUpdateItem],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if vaults.is_empty() {
            // // move on to next initialize operation
            // return self.execute_new_another_operation(ctx, None);
            return Ok((None, None));
        }
        let mut fund_account = ctx.fund_account.load_mut()?;
        let restaking_vault = fund_account.get_restaking_vault_mut(&vaults[0])?;

        let batch_size = items
            .len()
            .min(RESTAKING_VAULT_UPDATE_DELEGATIONS_BATCH_SIZE);

        let mut restaking_vault_delegation_update_items =
            Vec::<InitializeCommandResultRestakingVaultDelegationUpdate>::with_capacity(
                RESTAKING_VAULT_UPDATE_DELEGATIONS_BATCH_SIZE,
            );

        let receipt_token_pricing_source = restaking_vault
            .receipt_token_pricing_source
            .try_deserialize()?;
        match receipt_token_pricing_source {
            Some(TokenPricingSource::JitoRestakingVault { address }) => {
                let [vault_program, vault_config, vault_account, vault_update_state_tracker, remaining_accounts @ ..] =
                    accounts
                else {
                    err!(error::ErrorCode::AccountNotEnoughKeys)?
                };
                require_keys_eq!(address, vault_account.key());

                if remaining_accounts.len() < 2 * batch_size {
                    err!(error::ErrorCode::AccountNotEnoughKeys)?
                }
                let (accounts_to_update_delegation_state, _) =
                    remaining_accounts.split_at(2 * batch_size);

                let vault_service =
                    JitoRestakingVaultService::new(vault_program, vault_config, vault_account)?;

                vault_service.initialize_vault_update_state_tracker_if_needed(
                    ctx.system_program,
                    vault_update_state_tracker,
                    ctx.operator,
                    &[],
                )?;

                for (i, item) in items.iter().take(batch_size).enumerate() {
                    let vault_operator_delegation = accounts_to_update_delegation_state[2 * i];
                    let operator = accounts_to_update_delegation_state[2 * i + 1];
                    require_keys_eq!(operator.key(), item.operator);

                    let (delegated_amount, undelegation_requested_amount, undelegating_amount) =
                        vault_service.update_operator_delegation_state_if_needed(
                            vault_update_state_tracker,
                            vault_operator_delegation,
                            operator,
                            item.index,
                        )?;

                    // sync the state of the delegation
                    let delegation = restaking_vault.get_delegation_mut(&item.operator)?;
                    delegation.supported_token_delegated_amount = delegated_amount;
                    delegation.supported_token_undelegating_amount =
                        undelegation_requested_amount + undelegating_amount;

                    // store result items
                    restaking_vault_delegation_update_items.push(
                        InitializeCommandResultRestakingVaultDelegationUpdate {
                            operator: operator.key(),
                            delegated_amount: delegation.supported_token_delegated_amount,
                            undelegating_amount: delegation.supported_token_undelegating_amount,
                        },
                    );
                }

                let finalized = batch_size == items.len();
                if finalized {
                    vault_service.close_vault_update_state_tracker_if_needed(
                        vault_update_state_tracker,
                        ctx.operator,
                        &[],
                    )?;
                }

                let result = InitializeCommandResult {
                    restaking_vault_updated: Some(InitializeCommandResultRestakingVaultUpdated {
                        vault: restaking_vault.vault,
                        epoch: vault_service.get_current_epoch(),
                        finalized,
                        supported_token_mint: restaking_vault.supported_token_mint,
                        delegations: restaking_vault_delegation_update_items,
                    }),
                }
                .into();

                // move on to next delegations or vault or initialize operation
                if !finalized {
                    // move on to next delegations
                    let items = &items[batch_size..];
                    let accounts_to_update_vault_delegation_state =
                        vault_service.find_accounts_to_update_vault_delegation_state()?;
                    let accounts_to_update_operator_delegation_state = items
                        .iter()
                        .take(RESTAKING_VAULT_UPDATE_DELEGATIONS_BATCH_SIZE)
                        .flat_map(|item| {
                            vault_service
                                .find_accounts_to_update_operator_delegation_state(item.operator)
                        });
                    let required_accounts = accounts_to_update_vault_delegation_state
                        .chain(accounts_to_update_operator_delegation_state);
                    let entry = Self {
                        state: ExecuteRestakingVaultUpdate {
                            vaults: vaults.to_vec(),
                            items: items.to_vec(),
                        },
                    }
                    .with_required_accounts(required_accounts);

                    Ok((Some(result), Some(entry)))
                } else {
                    drop(fund_account);
                    // move on to next vault
                    if let Some(entry) = self
                        .create_prepare_restaking_vault_update_command(ctx, vaults[1..].to_vec())?
                    {
                        Ok((Some(result), Some(entry)))
                    } else {
                        // // move on to next initialize operation
                        // self.execute_new_another_operation(ctx, result)
                        Ok((Some(result), None))
                    }
                }
            }
            // otherwise fails
            Some(TokenPricingSource::SPLStakePool { .. })
            | Some(TokenPricingSource::MarinadeStakePool { .. })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        }
    }
}
