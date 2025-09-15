use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenAccount;

use crate::errors::ErrorCode;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::restaking::JitoRestakingVaultService;
use crate::modules::reward::{RewardAccount, UserRewardAccount};

use super::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct InitializeCommand {
    state: InitializeCommandState,
}

const RESTAKING_VAULT_UPDATE_DELEGATIONS_BATCH_SIZE: usize = 6;
const WRAPPED_TOKEN_UPDATE_BATCH_SIZE: usize = 12;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Default)]
pub enum InitializeCommandState {
    /// Initializes a command based on the fund state and strategy.
    #[default]
    New,
    /// Initializes restaking vault epoch process based on the fund state.
    NewRestakingVaultUpdate,
    /// Prepares to execute restaking vault epoch process.
    PrepareRestakingVaultUpdate { vault: Pubkey },
    /// Executes restaking vault epoch process and transitions to the next command,
    /// either preparing the next item or initializing wrapped token holder update process.
    ExecuteRestakingVaultUpdate {
        vault: Pubkey,
        /// Items could be empty.
        #[max_len(RESTAKING_VAULT_UPDATE_DELEGATIONS_BATCH_SIZE)]
        items: Vec<InitializeCommandRestakingVaultDelegationUpdateItem>,
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS)]
        next_item_indices: Vec<u64>,
    },
    /// Initializes wrapped token holder update process based on the fund state.
    NewWrappedTokenUpdate,
    /// Prepares to execute wrapped token holder update process.
    PrepareWrappedTokenUpdate {
        #[max_len(FUND_ACCOUNT_MAX_WRAPPED_TOKEN_HOLDERS)]
        wrapped_token_accounts: Vec<Pubkey>,
    },
    /// Executes wrapped token holder update process and transitions to the next command,
    /// either preparing the next item or performing a withdrawal operation.
    ExecuteWrappedTokenUpdate {
        #[max_len(FUND_ACCOUNT_MAX_WRAPPED_TOKEN_HOLDERS)]
        wrapped_token_accounts: Vec<Pubkey>,
    },
}

use InitializeCommandState::*;

impl core::fmt::Debug for InitializeCommandState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::New => write!(f, "New"),
            Self::NewRestakingVaultUpdate => write!(f, "NewRestakingVaultUpdate"),
            Self::PrepareRestakingVaultUpdate { vault } => f
                .debug_struct("PrepareRestakingVaultUpdate")
                .field("vault", vault)
                .finish(),
            Self::ExecuteRestakingVaultUpdate { vault, .. } => f
                .debug_struct("ExecuteRestakingVaultUpdate")
                .field("vault", vault)
                .finish(),
            Self::NewWrappedTokenUpdate => write!(f, "NewWrappedTokenUpdate"),
            Self::PrepareWrappedTokenUpdate { .. } => {
                f.debug_struct("PrepareWrappedTokenUpdate").finish()
            }
            Self::ExecuteWrappedTokenUpdate { .. } => {
                f.debug_struct("ExecuteWrappedTokenUpdate").finish()
            }
        }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize)]
pub struct InitializeCommandRestakingVaultDelegationUpdateItem {
    pub operator: Pubkey,
    pub index: u64,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct InitializeCommandResult {
    pub restaking_vault_updated: Option<InitializeCommandResultRestakingVaultUpdated>,
    pub wrapped_token_updated: Option<InitializeCommandResultWrappedTokenUpdated>,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct InitializeCommandResultRestakingVaultUpdated {
    pub vault: Pubkey,
    pub epoch: u64,
    pub finalized: bool,
    pub supported_token_mint: Pubkey,
    pub delegations: Vec<InitializeCommandResultRestakingVaultDelegationUpdate>,
}

impl From<InitializeCommandResultRestakingVaultUpdated> for OperationCommandResult {
    fn from(value: InitializeCommandResultRestakingVaultUpdated) -> Self {
        InitializeCommandResult {
            restaking_vault_updated: Some(value),
            ..Default::default()
        }
        .into()
    }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct InitializeCommandResultRestakingVaultDelegationUpdate {
    pub operator: Pubkey,
    pub delegated_amount: u64,
    pub undelegating_amount: u64,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct InitializeCommandResultWrappedTokenUpdated {
    pub receipt_token_mint: Pubkey,
    pub finalized: bool,
    pub wrapped_token_holders: Vec<InitializeCommandResultWrappedTokenHolderUpdate>,
}

impl From<InitializeCommandResultWrappedTokenUpdated> for OperationCommandResult {
    fn from(value: InitializeCommandResultWrappedTokenUpdated) -> Self {
        InitializeCommandResult {
            wrapped_token_updated: Some(value),
            ..Default::default()
        }
        .into()
    }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct InitializeCommandResultWrappedTokenHolderUpdate {
    pub wrapped_token_account: Pubkey,
    pub wrapped_token_amount: u64,
}

impl SelfExecutable for InitializeCommand {
    fn execute<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> ExecutionResult {
        let (result, entry) = match &self.state {
            // 1. restaking_vault_update
            New | NewRestakingVaultUpdate => {
                self.execute_new_restaking_vault_update_command(ctx, None, None)?
            }
            PrepareRestakingVaultUpdate { vault } => {
                self.execute_prepare_restaking_vault_update_command(ctx, accounts, vault)?
            }
            ExecuteRestakingVaultUpdate {
                vault,
                items,
                next_item_indices,
            } => self.execute_execute_restaking_vault_update_command(
                ctx,
                accounts,
                vault,
                items,
                next_item_indices,
            )?,
            // 2. wrapped_token_update
            NewWrappedTokenUpdate => self.execute_new_wrapped_token_update_command(ctx, None)?,
            PrepareWrappedTokenUpdate {
                wrapped_token_accounts,
            } => self.execute_prepare_wrapped_token_update_command(
                ctx,
                wrapped_token_accounts.clone(),
                None,
            )?,
            ExecuteWrappedTokenUpdate {
                wrapped_token_accounts,
            } => self.execute_execute_wrapped_token_update_command(
                ctx,
                accounts,
                wrapped_token_accounts,
            )?,
        };

        Ok((
            result,
            entry.or_else(|| {
                Some(EnqueueWithdrawalBatchCommand::default().without_required_accounts())
            }),
        ))
    }
}

impl InitializeCommand {
    #[inline(never)]
    fn execute_new_restaking_vault_update_command(
        &self,
        ctx: &OperationCommandContext,
        previous_vault: Option<&Pubkey>,
        previous_execution_result: Option<OperationCommandResult>,
    ) -> ExecutionResult {
        let fund_account = ctx.fund_account.load()?;
        let mut vaults_iter = fund_account
            .get_restaking_vaults_iter()
            .map(|restaking_vault| &restaking_vault.vault);
        let Some(vault) = (if let Some(previous_vault) = previous_vault {
            vaults_iter
                .skip_while(|vault| *vault != previous_vault)
                .nth(1)
        } else {
            vaults_iter.next()
        }) else {
            // fallback: 2. wrapped_token_update
            return self.execute_new_wrapped_token_update_command(ctx, previous_execution_result);
        };

        let receipt_token_pricing_source = fund_account
            .get_restaking_vault(vault)?
            .receipt_token_pricing_source
            .try_deserialize()?;
        let Some(entry) = (|| {
            let entry = match receipt_token_pricing_source {
                Some(TokenPricingSource::JitoRestakingVault { address }) => {
                    let required_accounts =
                        JitoRestakingVaultService::find_accounts_to_new(address)?;

                    let command = Self {
                        state: PrepareRestakingVaultUpdate { vault: *vault },
                    };
                    command.with_required_accounts(required_accounts)
                }
                Some(TokenPricingSource::VirtualVault { .. })
                | Some(TokenPricingSource::SolvBTCVault { .. }) => return Ok(None),
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
            };

            Result::Ok(Some(entry))
        })()?
        else {
            // fallback: next vault
            return self.execute_new_restaking_vault_update_command(
                ctx,
                Some(vault),
                previous_execution_result,
            );
        };

        Ok((previous_execution_result, Some(entry)))
    }

    #[inline(never)]
    fn execute_prepare_restaking_vault_update_command<'info>(
        &self,
        ctx: &OperationCommandContext,
        accounts: &[&'info AccountInfo<'info>],
        vault: &Pubkey,
    ) -> ExecutionResult {
        let fund_account = ctx.fund_account.load()?;
        let restaking_vault = fund_account.get_restaking_vault(vault)?;

        let receipt_token_pricing_source = restaking_vault
            .receipt_token_pricing_source
            .try_deserialize()?;
        let Some(entry) = (|| {
            let entry = match receipt_token_pricing_source {
                Some(TokenPricingSource::JitoRestakingVault { address }) => {
                    let [vault_program, vault_config, vault_account, ..] = accounts else {
                        err!(error::ErrorCode::AccountNotEnoughKeys)?
                    };
                    require_keys_eq!(address, vault_account.key());

                    let vault_service =
                        JitoRestakingVaultService::new(vault_program, vault_config, vault_account)?;

                    let next_item_indices = vault_service.get_ordered_vault_update_indices();

                    self.create_next_jito_restaking_vault_update_command(
                        &vault_service,
                        restaking_vault,
                        &next_item_indices,
                    )?
                }
                Some(TokenPricingSource::VirtualVault { .. })
                | Some(TokenPricingSource::SolvBTCVault { .. }) => {
                    return Ok(None);
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
                | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
                #[cfg(all(test, not(feature = "idl-build")))]
                Some(TokenPricingSource::Mock { .. }) => {
                    err!(ErrorCode::FundOperationCommandExecutionFailedException)?
                }
            };

            Result::Ok(Some(entry))
        })()?
        else {
            // fallback: next vault
            return self.execute_new_restaking_vault_update_command(ctx, Some(vault), None);
        };

        Ok((None, Some(entry)))
    }

    #[inline(never)]
    fn execute_execute_restaking_vault_update_command<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        vault: &Pubkey,
        items: &[InitializeCommandRestakingVaultDelegationUpdateItem],
        next_item_indices: &[u64],
    ) -> ExecutionResult {
        let mut fund_account = ctx.fund_account.load_mut()?;
        let restaking_vault = fund_account.get_restaking_vault_mut(vault)?;

        let mut result = Vec::with_capacity(RESTAKING_VAULT_UPDATE_DELEGATIONS_BATCH_SIZE);

        let receipt_token_pricing_source = restaking_vault
            .receipt_token_pricing_source
            .try_deserialize()?;
        let result = match receipt_token_pricing_source {
            Some(TokenPricingSource::JitoRestakingVault { address }) => {
                let [vault_program, vault_config, vault_account, vault_update_state_tracker, remaining_accounts @ ..] =
                    accounts
                else {
                    err!(error::ErrorCode::AccountNotEnoughKeys)?
                };
                require_keys_eq!(address, vault_account.key());

                if remaining_accounts.len() < 2 * items.len() {
                    err!(error::ErrorCode::AccountNotEnoughKeys)?
                }
                let (accounts_to_update_delegation_state, _) =
                    remaining_accounts.split_at(2 * items.len());

                let vault_service =
                    JitoRestakingVaultService::new(vault_program, vault_config, vault_account)?;

                vault_service.initialize_vault_update_state_tracker_if_needed(
                    ctx.system_program,
                    vault_update_state_tracker,
                    ctx.operator,
                    &[],
                )?;

                for (i, item) in items.iter().enumerate() {
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
                    result.push(InitializeCommandResultRestakingVaultDelegationUpdate {
                        operator: operator.key(),
                        delegated_amount: delegation.supported_token_delegated_amount,
                        undelegating_amount: delegation.supported_token_undelegating_amount,
                    });
                }

                let finalized = next_item_indices.is_empty();
                let result = InitializeCommandResultRestakingVaultUpdated {
                    vault: restaking_vault.vault,
                    epoch: vault_service.get_current_epoch(),
                    finalized,
                    supported_token_mint: restaking_vault.supported_token_mint,
                    delegations: result,
                }
                .into();

                if !finalized {
                    let entry = self.create_next_jito_restaking_vault_update_command(
                        &vault_service,
                        restaking_vault,
                        next_item_indices,
                    )?;

                    return Ok((Some(result), Some(entry)));
                }

                vault_service.close_vault_update_state_tracker_if_needed(
                    vault_update_state_tracker,
                    ctx.operator,
                    &[],
                )?;

                Some(result)
            }
            Some(TokenPricingSource::VirtualVault { .. })
            | Some(TokenPricingSource::SolvBTCVault { .. }) => None,
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
        };

        drop(fund_account);
        // move on to next vault
        self.execute_new_restaking_vault_update_command(ctx, Some(vault), result)
    }

    fn create_next_jito_restaking_vault_update_command(
        &self,
        vault_service: &JitoRestakingVaultService,
        restaking_vault: &RestakingVault,
        next_item_indices: &[u64],
    ) -> Result<OperationCommandEntry> {
        // find next batch
        let batch_size = next_item_indices
            .len()
            .min(RESTAKING_VAULT_UPDATE_DELEGATIONS_BATCH_SIZE);
        let mut items = Vec::with_capacity(RESTAKING_VAULT_UPDATE_DELEGATIONS_BATCH_SIZE);
        for &index in next_item_indices.iter().take(batch_size) {
            let operator = restaking_vault
                .get_delegation_by_index(index as usize)?
                .operator;
            items.push(InitializeCommandRestakingVaultDelegationUpdateItem { operator, index });
        }
        let next_item_indices = next_item_indices[batch_size..].to_vec();

        // create next command entry
        let accounts_to_update_vault_delegation_state =
            vault_service.find_accounts_to_update_vault_state()?;
        let accounts_to_update_operator_delegation_state = items
            .iter()
            .map(|item| item.operator)
            .collect::<Vec<_>>()
            .into_iter()
            .flat_map(|operator| vault_service.find_accounts_to_update_delegation_state(operator));
        let required_accounts = accounts_to_update_vault_delegation_state
            .chain(accounts_to_update_operator_delegation_state);

        let command = Self {
            state: ExecuteRestakingVaultUpdate {
                vault: restaking_vault.vault,
                items,
                next_item_indices,
            },
        };
        let entry = command.with_required_accounts(required_accounts);

        Ok(entry)
    }

    #[inline(never)]
    fn execute_new_wrapped_token_update_command(
        &self,
        ctx: &OperationCommandContext,
        previous_execution_result: Option<OperationCommandResult>,
    ) -> ExecutionResult {
        let fund_account = ctx.fund_account.load()?;
        let Some(wrapped_token) = fund_account.get_wrapped_token() else {
            return Ok((previous_execution_result, None));
        };

        let mut wrapped_token_accounts = wrapped_token
            .get_holders_iter()
            .map(|holder| holder.token_account)
            .collect::<Vec<_>>();
        if !wrapped_token_accounts.is_empty() {
            // pseudo-randomize the order
            let clock = Clock::get()?;
            let start_idx = clock.slot as usize % wrapped_token_accounts.len();
            wrapped_token_accounts.rotate_left(start_idx);
        }

        self.execute_prepare_wrapped_token_update_command(
            ctx,
            wrapped_token_accounts,
            previous_execution_result,
        )
    }

    #[inline(never)]
    fn execute_prepare_wrapped_token_update_command(
        &self,
        ctx: &OperationCommandContext,
        wrapped_token_accounts: Vec<Pubkey>,
        previous_execution_result: Option<OperationCommandResult>,
    ) -> ExecutionResult {
        if wrapped_token_accounts.is_empty() {
            return Ok((previous_execution_result, None));
        }

        let fund_account = ctx.fund_account.load()?;

        let reward_account = RewardAccount::find_account_address(&ctx.receipt_token_mint.key());
        let fund_wrap_account = fund_account.get_wrap_account_address()?;
        let fund_wrap_account_reward_account = UserRewardAccount::find_account_address(
            &ctx.receipt_token_mint.key(),
            &fund_wrap_account,
        );
        let accounts_to_update_holders = wrapped_token_accounts
            .iter()
            .copied()
            .take(WRAPPED_TOKEN_UPDATE_BATCH_SIZE)
            .collect::<Vec<_>>()
            .into_iter()
            .flat_map(|wrapped_token_account| {
                let wrapped_token_holder_reward_account = UserRewardAccount::find_account_address(
                    &ctx.receipt_token_mint.key(),
                    &wrapped_token_account,
                );
                [
                    (wrapped_token_account, false),
                    (wrapped_token_holder_reward_account, true),
                ]
            });
        let required_accounts = [
            (reward_account, true),
            (fund_wrap_account, false),
            (fund_wrap_account_reward_account, true),
        ]
        .into_iter()
        .chain(accounts_to_update_holders);

        let command = Self {
            state: ExecuteWrappedTokenUpdate {
                wrapped_token_accounts,
            },
        };
        let entry = command.with_required_accounts(required_accounts);

        Ok((previous_execution_result, Some(entry)))
    }

    #[inline(never)]
    fn execute_execute_wrapped_token_update_command<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        wrapped_token_accounts: &[Pubkey],
    ) -> ExecutionResult {
        if wrapped_token_accounts.is_empty() {
            return Ok((None, None));
        }
        let batch_size = wrapped_token_accounts
            .len()
            .min(WRAPPED_TOKEN_UPDATE_BATCH_SIZE);

        let [reward_account, fund_wrap_account, fund_wrap_account_reward_account, remaining_account @ ..] =
            accounts
        else {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        };

        if remaining_account.len() < 2 * batch_size {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        }
        let (accounts_to_update_holder, _) = remaining_account.split_at(2 * batch_size);

        let fund_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?;

        let reward_account = AccountLoader::<RewardAccount>::try_from(reward_account)?;
        let fund_wrap_account = SystemAccount::try_from(fund_wrap_account)?;
        let fund_wrap_account_reward_account =
            AccountLoader::<UserRewardAccount>::try_from(fund_wrap_account_reward_account)?;

        let mut result = Vec::with_capacity(WRAPPED_TOKEN_UPDATE_BATCH_SIZE);
        for (i, wrapped_token_account) in wrapped_token_accounts.iter().take(batch_size).enumerate()
        {
            let wrapped_token_holder = accounts_to_update_holder[2 * i];
            let wrapped_token_holder_reward_account = accounts_to_update_holder[2 * i + 1];
            require_keys_eq!(wrapped_token_holder.key(), *wrapped_token_account);

            let wrapped_token_holder =
                InterfaceAccount::<TokenAccount>::try_from(wrapped_token_holder)?;
            let wrapped_token_holder_reward_account =
                AccountLoader::<UserRewardAccount>::try_from(wrapped_token_holder_reward_account)?;

            fund_service.update_wrapped_token_holder(
                &reward_account,
                &fund_wrap_account,
                &fund_wrap_account_reward_account,
                &wrapped_token_holder,
                &wrapped_token_holder_reward_account,
            )?;

            result.push(InitializeCommandResultWrappedTokenHolderUpdate {
                wrapped_token_account: wrapped_token_holder.key(),
                wrapped_token_amount: wrapped_token_holder.amount,
            })
        }

        drop(fund_service);

        let finalized = batch_size == wrapped_token_accounts.len();
        let result = InitializeCommandResultWrappedTokenUpdated {
            receipt_token_mint: ctx.receipt_token_mint.key(),
            finalized,
            wrapped_token_holders: result,
        }
        .into();

        if !finalized {
            return self.execute_prepare_wrapped_token_update_command(
                ctx,
                wrapped_token_accounts[batch_size..].to_vec(),
                Some(result),
            );
        }

        Ok((Some(result), None))
    }
}
