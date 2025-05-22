use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::pricing::{PricingService, TokenPricingSource};
use crate::modules::restaking::JitoRestakingVaultService;
use crate::utils::PDASeeds;

use super::{
    OperationCommandContext, OperationCommandEntry, OperationCommandResult, SelfExecutable,
    WeightedAllocationParticipant, WeightedAllocationStrategy, FUND_ACCOUNT_MAX_RESTAKING_VAULTS,
    FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct DelegateVSTCommand {
    state: DelegateVSTCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct DelegateVSTCommandItem {
    operator: Pubkey,
    allocated_supported_token_amount: u64,
}

const RESTAKING_VAULT_DELEGATE_BATCH_SIZE: usize = 6;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum DelegateVSTCommandState {
    #[default]
    New,
    Prepare {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        vaults: Vec<Pubkey>,
    },
    Execute {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        vaults: Vec<Pubkey>,

        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS)]
        items: Vec<DelegateVSTCommandItem>,
    },
}

use DelegateVSTCommandState::*;

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct DelegateVSTCommandResult {
    pub vault: Pubkey,
    pub delegations: Vec<DelegateVSTCommandResultDelegated>,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct DelegateVSTCommandResultDelegated {
    pub operator: Pubkey,
    pub delegated_token_amount: u64,
    pub total_delegated_token_amount: u64,
}

impl SelfExecutable for DelegateVSTCommand {
    fn execute<'a, 'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let (result, entry) = match &self.state {
            New => self.execute_new(ctx)?,
            Prepare { vaults } => self.execute_prepare(ctx, accounts, vaults)?,
            Execute { vaults, items } => self.execute_execute(ctx, accounts, vaults, items)?,
        };

        Ok((result, entry))
    }
}

#[deny(clippy::wildcard_enum_match_arm)]
impl DelegateVSTCommand {
    #[inline(never)]
    fn execute_new<'info>(
        &self,
        ctx: &OperationCommandContext<'info, '_>,
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let fund_account = ctx.fund_account.load()?;
        let mut vaults = Vec::with_capacity(FUND_ACCOUNT_MAX_RESTAKING_VAULTS);
        for restaking_vault in fund_account.get_restaking_vaults_iter() {
            vaults.push(restaking_vault.vault);
        }

        Ok((None, self.create_prepare_command(ctx, vaults)?))
    }

    fn create_prepare_command<'info>(
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
            state: Prepare { vaults },
        };
        let entry = match receipt_token_pricing_source {
            Some(TokenPricingSource::JitoRestakingVault { address }) => {
                let required_accounts = JitoRestakingVaultService::find_accounts_to_new(address)?;
                command.with_required_accounts(required_accounts)
            }
            Some(TokenPricingSource::SolvBTCVault { .. })
            | Some(TokenPricingSource::VirtualRestakingVault { .. }) => {
                // TODO/v0.7.0: deal with solv vault if needed
                command.without_required_accounts()
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

        Ok(Some(entry))
    }

    #[inline(never)]
    fn execute_prepare<'info>(
        &self,
        ctx: &OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        vaults: &[Pubkey],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if vaults.is_empty() {
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
                let mut items = Vec::with_capacity(FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS);
                let mut strategy = WeightedAllocationStrategy::<
                    FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS,
                >::new(
                    restaking_vault.get_delegations_iter().map(|delegation| {
                        items.push(DelegateVSTCommandItem {
                            operator: delegation.operator,
                            allocated_supported_token_amount: 0,
                        });

                        WeightedAllocationParticipant::new(
                            delegation.supported_token_allocation_weight,
                            delegation.supported_token_delegated_amount,
                            delegation.supported_token_allocation_capacity_amount,
                        )
                    }),
                );
                strategy.put(vault_service.get_available_amount_to_delegate()?)?;

                const MIN_ALLOCATED_TOKEN_AMOUNT: u64 = 1_000_000_000;
                for (index, _) in strategy.get_participants_iter().enumerate() {
                    let allocated_token_amount =
                        strategy.get_participant_last_put_amount_by_index(index)?;

                    if allocated_token_amount >= MIN_ALLOCATED_TOKEN_AMOUNT {
                        items[index].allocated_supported_token_amount = allocated_token_amount;
                    }
                }
                items.retain(|item| {
                    item.allocated_supported_token_amount >= MIN_ALLOCATED_TOKEN_AMOUNT
                });

                if items.is_empty() {
                    // move on to next vault
                    let vaults = vaults[1..].to_vec();
                    return Ok((None, self.create_prepare_command(ctx, vaults)?));
                }

                let operators = items
                    .iter()
                    .take(RESTAKING_VAULT_DELEGATE_BATCH_SIZE)
                    .map(|item| item.operator)
                    .collect::<Vec<_>>();
                let accounts_to_new = JitoRestakingVaultService::find_accounts_to_new(address)?;
                let accounts_to_delegate = operators.iter().flat_map(|operator| {
                    vault_service.find_accounts_to_update_delegation_state(*operator)
                });

                let required_accounts = accounts_to_new.chain(accounts_to_delegate);
                let entry = Self {
                    state: Execute {
                        vaults: vaults.to_vec(),
                        items,
                    },
                }
                .with_required_accounts(required_accounts);

                Ok((None, Some(entry)))
            }
            Some(TokenPricingSource::SolvBTCVault { .. })
            | Some(TokenPricingSource::VirtualRestakingVault { .. }) => {
                // TODO/v0.7.0: deal with solv vault if needed
                Ok((
                    None,
                    self.create_prepare_command(ctx, vaults[1..].to_vec())?,
                ))
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
            // move on to next vault
            let vaults = vaults[1..].to_vec();
            return Ok((None, self.create_prepare_command(ctx, vaults)?));
        }

        let batch_size = items.len().min(RESTAKING_VAULT_DELEGATE_BATCH_SIZE);
        let mut delegation_results = Vec::with_capacity(RESTAKING_VAULT_DELEGATE_BATCH_SIZE);

        let fund_account = ctx.fund_account.load()?;
        let receipt_token_pricing_source = fund_account
            .get_restaking_vault(&vaults[0])?
            .receipt_token_pricing_source
            .try_deserialize()?;
        match receipt_token_pricing_source {
            Some(TokenPricingSource::JitoRestakingVault { address }) => {
                let [vault_program, vault_config, vault_account, remaining_accounts @ ..] =
                    accounts
                else {
                    err!(error::ErrorCode::AccountNotEnoughKeys)?
                };
                require_keys_eq!(address, vault_account.key());

                if remaining_accounts.len() < 2 * batch_size {
                    err!(error::ErrorCode::AccountNotEnoughKeys)?
                }
                let (accounts_to_delegate, _) = remaining_accounts.split_at(2 * batch_size);

                let vault_service =
                    JitoRestakingVaultService::new(vault_program, vault_config, vault_account)?;

                for (i, item) in items.iter().take(batch_size).enumerate() {
                    let vault_operator_delegation = accounts_to_delegate[2 * i];
                    let operator = accounts_to_delegate[2 * i + 1];
                    require_keys_eq!(operator.key(), item.operator);

                    vault_service.add_delegation(
                        vault_operator_delegation,
                        operator,
                        ctx.fund_account.as_ref(),
                        &[fund_account.get_seeds().as_ref()],
                        item.allocated_supported_token_amount,
                    )?;
                }

                drop(fund_account);
                let mut fund_account = ctx.fund_account.load_mut()?;
                let restaking_vault = fund_account.get_restaking_vault_mut(&vaults[0])?;
                for (i, item) in items.iter().take(batch_size).enumerate() {
                    let operator = accounts_to_delegate[2 * i + 1];
                    let delegation = restaking_vault.get_delegation_mut(operator.key)?;
                    delegation.supported_token_delegated_amount +=
                        item.allocated_supported_token_amount;
                    delegation_results.push(DelegateVSTCommandResultDelegated {
                        operator: operator.key(),
                        delegated_token_amount: item.allocated_supported_token_amount,
                        total_delegated_token_amount: delegation.supported_token_delegated_amount,
                    });
                }

                let result = DelegateVSTCommandResult {
                    vault: vaults[0],
                    delegations: delegation_results,
                }
                .into();

                let finalized = batch_size == items.len();
                if !finalized {
                    // move on to next delegations
                    let items = &items[batch_size..];
                    let accounts_to_new =
                        JitoRestakingVaultService::find_accounts_to_new(vaults[0])?;
                    let accounts_to_delegate = items
                        .iter()
                        .take(RESTAKING_VAULT_DELEGATE_BATCH_SIZE)
                        .flat_map(|item| {
                            vault_service.find_accounts_to_update_delegation_state(item.operator)
                        });
                    let required_accounts = accounts_to_new.chain(accounts_to_delegate);
                    let entry = Self {
                        state: Execute {
                            vaults: vaults.to_vec(),
                            items: items.to_vec(),
                        },
                    }
                    .with_required_accounts(required_accounts);

                    Ok((Some(result), Some(entry)))
                } else {
                    drop(fund_account);
                    // move on to next vault
                    let vaults = vaults[1..].to_vec();
                    Ok((Some(result), self.create_prepare_command(ctx, vaults)?))
                }
            }
            Some(TokenPricingSource::SolvBTCVault { .. })
            | Some(TokenPricingSource::VirtualRestakingVault { .. }) => {
                // TODO/v0.7.0: deal with solv vault if needed
                Ok((
                    None,
                    self.create_prepare_command(ctx, vaults[1..].to_vec())?,
                ))
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
        }
    }
}
