use anchor_lang::prelude::*;

use crate::errors;
use crate::modules::fund::fund_account_restaking_vault::RestakingVault;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::restaking::JitoRestakingVaultService;

use super::{
    EnqueueWithdrawalBatchCommand, OperationCommand, OperationCommandContext,
    OperationCommandEntry, OperationCommandResult, RestakeVSTCommand, RestakeVSTCommandState,
    SelfExecutable, FUND_ACCOUNT_MAX_RESTAKING_VAULTS,
    FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct InitializeCommand {
    state: InitializeCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum InitializeCommandState {
    /// Initializes a command with items based on the fund state and strategy.
    #[default]
    New,
    /// Prepares to execute restaking vault epoch process for a single item, intended to be used as a manual reset command.
    PrepareSingleRestakingVaultUpdate { vault: Pubkey, operator: Pubkey },
    /// Prepares to execute restaking vault epoch process for the first item in the list.
    PrepareRestakingVaultUpdate {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        items: Vec<InitializeCommandRestakingVaultUpdateItem>,
    },
    /// Executes restaking vault epoch process for the first item and transitions to the next command,
    /// either preparing the next item or performing a withdrawal operation.
    ExecuteRestakingVaultUpdate {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        items: Vec<InitializeCommandRestakingVaultUpdateItem>,
    },
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct InitializeCommandRestakingVaultUpdateItem {
    pub vault: Pubkey,
    #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS)]
    pub delegations_updated_bitmap: Vec<bool>,
}

const RESTAKING_VAULT_UPDATE_DELEGATIONS_BATCH_SIZE: usize = 5;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct InitializeCommandResult {
    pub restaking_vault_updated: Option<InitializeCommandResultRestakingVaultUpdated>,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct InitializeCommandResultRestakingVaultUpdated {
    pub vault: Pubkey,
    pub epoch: u64,
    pub finalized: bool,
    pub supported_token_mint: Pubkey,
    #[max_len(RESTAKING_VAULT_UPDATE_DELEGATIONS_BATCH_SIZE)]
    pub delegations: Vec<InitializeCommandResultRestakingVaultDelegationUpdate>,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
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
        let mut result: Option<OperationCommandResult> = None;
        let mut restaking_vault_update_items: Option<
            Vec<InitializeCommandRestakingVaultUpdateItem>,
        > = None;

        match &self.state {
            InitializeCommandState::New => {
                let fund_account = ctx.fund_account.load()?;

                // prepare restaking vault update process which shall be processed for each epoch
                let items = fund_account
                    .get_restaking_vaults_iter()
                    .map(|restaking_vault| {
                        Ok(
                            match restaking_vault
                                .receipt_token_pricing_source
                                .try_deserialize()?
                            {
                                Some(TokenPricingSource::JitoRestakingVault { .. }) => {
                                    InitializeCommandRestakingVaultUpdateItem {
                                        vault: restaking_vault.vault,
                                        delegations_updated_bitmap: restaking_vault
                                            .get_delegations_iter()
                                            .map(|_| false)
                                            .collect::<Vec<_>>(),
                                    }
                                }
                                _ => err!(
                                    errors::ErrorCode::FundOperationCommandExecutionFailedException
                                )?,
                            },
                        )
                    })
                    .collect::<Result<Vec<_>>>()?;

                if items.len() > 0 {
                    restaking_vault_update_items = Some(items);
                }
            }
            InitializeCommandState::PrepareSingleRestakingVaultUpdate { vault, operator } => {
                let fund_account = ctx.fund_account.load()?;
                let restaking_vault = fund_account.get_restaking_vault(vault)?;
                restaking_vault.get_delegation(operator)?; // check existence

                restaking_vault_update_items =
                    Some(vec![InitializeCommandRestakingVaultUpdateItem {
                        vault: *vault,
                        delegations_updated_bitmap: restaking_vault
                            .get_delegations_iter()
                            .map(|delegation| delegation.operator != *operator)
                            .collect::<Vec<_>>(),
                    }]);
            }
            InitializeCommandState::PrepareRestakingVaultUpdate { items } => {
                match items.first() {
                    Some(item) => {
                        let fund_account = ctx.fund_account.load()?;
                        let restaking_vault = fund_account.get_restaking_vault(&item.vault)?;
                        let required_accounts = match restaking_vault
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

                                let vault_service = JitoRestakingVaultService::new(
                                    vault_program,
                                    vault_config,
                                    vault_account,
                                )?;
                                let mut required_accounts =
                                    vault_service.find_account_to_ensure_state_update_required()?;

                                // append part of unprocessed operator accounts
                                required_accounts.extend(
                                    restaking_vault
                                        .get_delegations_iter()
                                        .take(item.delegations_updated_bitmap.len())
                                        .enumerate()
                                        .filter(|(index, _)| {
                                            !item.delegations_updated_bitmap.get(*index).unwrap()
                                        })
                                        .take(RESTAKING_VAULT_UPDATE_DELEGATIONS_BATCH_SIZE)
                                        .flat_map(|(_, delegation)| {
                                            [
                                                (
                                                    vault_service
                                                        .find_vault_operator_delegation_address(
                                                            &delegation.operator,
                                                        ),
                                                    true,
                                                ),
                                                (delegation.operator, false),
                                            ]
                                        }),
                                );

                                required_accounts
                            }
                            _ => err!(
                                errors::ErrorCode::FundOperationCommandExecutionFailedException
                            )?,
                        };

                        return Ok((
                            None,
                            Some(
                                InitializeCommand {
                                    state: InitializeCommandState::ExecuteRestakingVaultUpdate {
                                        items: items.clone(),
                                    },
                                }
                                .with_required_accounts(required_accounts),
                            ),
                        ));
                    }
                    None => {}
                }
            }

            InitializeCommandState::ExecuteRestakingVaultUpdate { items } => {
                match items.first() {
                    Some(item) => {
                        let mut remaining_items =
                            items.into_iter().skip(1).cloned().collect::<Vec<_>>();

                        let fund_account = ctx.fund_account.load()?;
                        let restaking_vault = fund_account.get_restaking_vault(&item.vault)?;
                        match restaking_vault
                            .receipt_token_pricing_source
                            .try_deserialize()?
                        {
                            Some(TokenPricingSource::JitoRestakingVault { address }) => {
                                let [vault_program, vault_config, vault_account, system_program, vault_update_state_tracker1, vault_update_state_tracker2, remaining_accounts @ ..] =
                                    accounts
                                else {
                                    err!(ErrorCode::AccountNotEnoughKeys)?
                                };
                                require_keys_eq!(address, vault_account.key());

                                let vault_service = JitoRestakingVaultService::new(
                                    vault_program,
                                    vault_config,
                                    vault_account,
                                )?;

                                let update_required_state_tracker = vault_service
                                    .ensure_state_update_required(
                                        system_program,
                                        vault_update_state_tracker1,
                                        vault_update_state_tracker2,
                                        ctx.operator,
                                        &[],
                                    )?;

                                if let Some(vault_update_state_tracker) =
                                    update_required_state_tracker
                                {
                                    // prepare required accounts for update required delegations.
                                    let processing_delegations = restaking_vault
                                        .get_delegations_iter()
                                        .take(item.delegations_updated_bitmap.len())
                                        .enumerate()
                                        .filter(|(index, _)| {
                                            !item.delegations_updated_bitmap.get(*index).unwrap()
                                        })
                                        .take(RESTAKING_VAULT_UPDATE_DELEGATIONS_BATCH_SIZE)
                                        .flat_map(|(bitmap_index, delegation)| {
                                            match remaining_accounts.iter().position(|account| {
                                                delegation.operator == account.key()
                                            }) {
                                                Some(account_index) => Some((
                                                    bitmap_index,
                                                    remaining_accounts[account_index - 1],
                                                    remaining_accounts[account_index],
                                                )),
                                                None => None,
                                            }
                                        })
                                        .collect::<Vec<_>>();

                                    drop(fund_account);

                                    let mut fund_account = ctx.fund_account.load_mut()?;
                                    let restaking_vault =
                                        fund_account.get_restaking_vault_mut(&item.vault)?;

                                    let mut restaking_vault_delegation_update_items = Vec::<
                                        InitializeCommandResultRestakingVaultDelegationUpdate,
                                    >::with_capacity(
                                        RESTAKING_VAULT_UPDATE_DELEGATIONS_BATCH_SIZE,
                                    );

                                    // update target delegations one by one.
                                    let mut processing_item = item.clone();
                                    for (bitmap_index, vault_operator_delegation, operator) in
                                        processing_delegations
                                    {
                                        let (
                                            restaked_amount,
                                            undelegation_requested_amount,
                                            undelegating_amount,
                                        ) = vault_service.update_delegation_state(
                                            vault_update_state_tracker,
                                            vault_operator_delegation,
                                            operator,
                                            ctx.operator,
                                            &[],
                                        )?;
                                        processing_item.delegations_updated_bitmap[bitmap_index] =
                                            true;

                                        // sync the state of the delegation
                                        let delegation =
                                            restaking_vault.get_delegation_mut(&operator.key())?;
                                        delegation.supported_token_delegated_amount =
                                            restaked_amount;
                                        delegation.supported_token_undelegating_amount =
                                            undelegation_requested_amount + undelegating_amount;

                                        // store result items
                                        restaking_vault_delegation_update_items.push(
                                            InitializeCommandResultRestakingVaultDelegationUpdate {
                                                operator: operator.key(),
                                                delegated_amount: delegation
                                                    .supported_token_delegated_amount,
                                                undelegating_amount: delegation
                                                    .supported_token_undelegating_amount,
                                            },
                                        );
                                    }

                                    let mut finalized = false;
                                    if processing_item
                                        .delegations_updated_bitmap
                                        .iter()
                                        .all(|b| *b)
                                    {
                                        // finalize the update if all delegations have been updated.
                                        finalized = vault_service
                                            .ensure_state_update_required(
                                                system_program,
                                                vault_update_state_tracker,
                                                vault_update_state_tracker,
                                                ctx.operator,
                                                &[],
                                            )?
                                            .is_none();
                                    } else {
                                        // push the item back as the process not completed yet
                                        remaining_items.insert(0, processing_item)
                                    }

                                    // store the result
                                    result = Some(
                                        InitializeCommandResult {
                                            restaking_vault_updated: Some(
                                                InitializeCommandResultRestakingVaultUpdated {
                                                    vault: restaking_vault.vault,
                                                    epoch: vault_service.get_current_epoch(),
                                                    finalized,
                                                    supported_token_mint: restaking_vault
                                                        .supported_token_mint,
                                                    delegations:
                                                        restaking_vault_delegation_update_items,
                                                },
                                            ),
                                        }
                                        .into(),
                                    );
                                }

                                restaking_vault_update_items = Some(remaining_items);
                            }
                            _ => err!(
                                errors::ErrorCode::FundOperationCommandExecutionFailedException
                            )?,
                        };
                    }
                    None => {}
                }
            }
        }

        // transition to next command
        Ok((
            result,
            Some({
                if let Some(items) = restaking_vault_update_items {
                    match items.first() {
                        Some(item) => {

                            let fund_account = ctx.fund_account.load()?;
                            let restaking_vault = fund_account.get_restaking_vault(&item.vault)?;
                            let required_accounts = match restaking_vault
                                .receipt_token_pricing_source
                                .try_deserialize()?
                            {
                                Some(TokenPricingSource::JitoRestakingVault { .. }) => {
                                    JitoRestakingVaultService::find_accounts_to_new(item.vault)?
                                }
                                _ => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
                            };

                            Some(
                                InitializeCommand {
                                    state:
                                    InitializeCommandState::PrepareRestakingVaultUpdate {
                                        items,
                                    },
                                }
                                    .with_required_accounts(required_accounts),
                            )
                        }
                        None => None
                    }
                } else {
                    None
                }
            }.unwrap_or_else(|| EnqueueWithdrawalBatchCommand::default().without_required_accounts())),
        ))
    }
}
