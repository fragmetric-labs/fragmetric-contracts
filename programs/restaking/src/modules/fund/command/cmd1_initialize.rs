use anchor_lang::prelude::*;

use crate::errors;
use crate::modules::fund::fund_account_restaking_vault::RestakingVault;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::restaking::JitoRestakingVaultService;

use super::{
    EnqueueWithdrawalBatchCommand, OperationCommand, OperationCommandContext,
    OperationCommandEntry, OperationCommandResult, RestakeVSTCommand, RestakeVSTCommandState,
    SelfExecutable, FUND_ACCOUNT_MAX_RESTAKING_VAULTS, FUND_ACCOUNT_MAX_RESTAKING_VAULT_OPERATORS,
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
    PrepareSingleRestakingVaultEpochProcess { vault: Pubkey, operator: Pubkey },
    /// Prepares to execute restaking vault epoch process for the first item in the list.
    PrepareRestakingVaultEpochProcess {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        items: Vec<InitializeCommandRestakingVaultEpochProcessItem>,
    },
    /// Executes restaking vault epoch process for the first item and transitions to the next command,
    /// either preparing the next item or performing a withdrawal operation.
    ExecuteRestakingVaultEpochProcess {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        items: Vec<InitializeCommandRestakingVaultEpochProcessItem>,
    },
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct InitializeCommandRestakingVaultEpochProcessItem {
    vault: Pubkey,
    #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULT_OPERATORS)]
    operators_processed_bitmap: Vec<bool>,
}

const RESTAKING_VAULT_EPOCH_PROCESS_BATCH_SIZE: usize = 5;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct InitializeCommandResult {
    restaking_vault_epoch_processed: Option<InitializeCommandResultRestakingVaultEpochProcessed>,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct InitializeCommandResultRestakingVaultEpochProcessed {
    vault: Pubkey,
    supported_token_mint: Pubkey,
    #[max_len(RESTAKING_VAULT_EPOCH_PROCESS_BATCH_SIZE)]
    items: Vec<InitializeCommandResultRestakingVaultEpochProcessedItem>,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct InitializeCommandResultRestakingVaultEpochProcessedItem {
    pub operator: Pubkey,
    pub restaked_amount: u64,
    pub undelegation_requested_amount: u64,
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
        let mut restaking_vault_epoch_process_items: Option<
            Vec<InitializeCommandRestakingVaultEpochProcessItem>,
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
                                    InitializeCommandRestakingVaultEpochProcessItem {
                                        vault: restaking_vault.vault,
                                        operators_processed_bitmap: restaking_vault
                                            .get_operators_iter()
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
                    restaking_vault_epoch_process_items = Some(items);
                }
            }
            InitializeCommandState::PrepareSingleRestakingVaultEpochProcess { vault, operator } => {
                let fund_account = ctx.fund_account.load()?;
                let restaking_vault = fund_account.get_restaking_vault(vault)?;
                restaking_vault.get_operator(operator)?; // check existence

                restaking_vault_epoch_process_items =
                    Some(vec![InitializeCommandRestakingVaultEpochProcessItem {
                        vault: *vault,
                        operators_processed_bitmap: restaking_vault
                            .get_operators_iter()
                            .map(|o| o.operator != *operator)
                            .collect::<Vec<_>>(),
                    }]);
            }
            InitializeCommandState::PrepareRestakingVaultEpochProcess { items } => {
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
                                        .get_operators_iter()
                                        .take(item.operators_processed_bitmap.len())
                                        .enumerate()
                                        .filter(|(index, _)| {
                                            !item.operators_processed_bitmap.get(*index).unwrap()
                                        })
                                        .take(RESTAKING_VAULT_EPOCH_PROCESS_BATCH_SIZE)
                                        .flat_map(|(_, o)| {
                                            [
                                                (
                                                    vault_service
                                                        .find_vault_operator_delegation_address(
                                                            &o.operator,
                                                        ),
                                                    true,
                                                ),
                                                (o.operator, false),
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
                                    state:
                                        InitializeCommandState::ExecuteRestakingVaultEpochProcess {
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

            InitializeCommandState::ExecuteRestakingVaultEpochProcess { items } => {
                match items.first() {
                    Some(item) => {
                        let fund_account = ctx.fund_account.load()?;
                        let restaking_vault = fund_account.get_restaking_vault(&item.vault)?;
                        let required_accounts = match restaking_vault
                            .receipt_token_pricing_source
                            .try_deserialize()?
                        {
                            Some(TokenPricingSource::JitoRestakingVault { address }) => {
                                let [vault_program, vault_config, vault_account, system_program, vault_update_state_tracker, remaining_accounts @ ..] =
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
                                let mut state_update_required = vault_service
                                    .ensure_state_update_required(
                                        system_program,
                                        vault_update_state_tracker,
                                        ctx.operator,
                                        &[],
                                    )?;

                                if state_update_required {
                                    let processing_operators = restaking_vault
                                        .get_operators_iter()
                                        .take(item.operators_processed_bitmap.len())
                                        .enumerate()
                                        .filter(|(index, o)| {
                                            !item.operators_processed_bitmap.get(*index).unwrap()
                                        })
                                        .flat_map(|(bitmap_index, o)| {
                                            match remaining_accounts
                                                .iter()
                                                .position(|account| o.operator == account.key())
                                            {
                                                Some(account_index) => Some((
                                                    bitmap_index,
                                                    remaining_accounts[account_index - 1],
                                                    remaining_accounts[account_index],
                                                )),
                                                None => None,
                                            }
                                        })
                                        .collect::<Vec<_>>();

                                    let mut fund_account = ctx.fund_account.load_mut()?;
                                    let mut restaking_vault =
                                        fund_account.get_restaking_vault_mut(&item.vault)?;

                                    let mut restaking_vault_epoch_processed_items = Vec::<
                                        InitializeCommandResultRestakingVaultEpochProcessedItem,
                                    >::new(
                                    );

                                    let mut updated_item = item.clone();
                                    for (bitmap_index, vault_operator_delegation, operator) in
                                        processing_operators
                                    {
                                        let (
                                            restaked_amount,
                                            undelegation_requested_amount,
                                            undelegating_amount,
                                        ) = vault_service
                                            .update_operator_delegation_state_if_needed(
                                                vault_update_state_tracker,
                                                vault_operator_delegation,
                                                operator,
                                                ctx.operator,
                                                &[],
                                            )?;

                                        let restaking_vault_operator =
                                            restaking_vault.get_operator_mut(&operator.key())?;
                                        restaking_vault_operator.supported_token_delegated_amount =
                                            restaked_amount
                                                + undelegation_requested_amount
                                                + undelegating_amount;

                                        updated_item.operators_processed_bitmap[bitmap_index] =
                                            true;

                                        restaking_vault_epoch_processed_items.push(
                                            InitializeCommandResultRestakingVaultEpochProcessedItem {
                                                operator: operator.key(),
                                                restaked_amount,
                                                undelegation_requested_amount,
                                                undelegating_amount,
                                            }
                                        );
                                    }

                                    if updated_item.operators_processed_bitmap.iter().all(|b| *b) {
                                        state_update_required = vault_service
                                            .ensure_state_update_required(
                                                system_program,
                                                vault_update_state_tracker,
                                                ctx.operator,
                                                &[],
                                            )?;
                                    }

                                    if state_update_required {
                                        let mut remaining_items =
                                            items.into_iter().skip(1).cloned().collect::<Vec<_>>();
                                        remaining_items.insert(0, updated_item);
                                        restaking_vault_epoch_process_items = Some(remaining_items);
                                    }

                                    result = Some(InitializeCommandResult {
                                        restaking_vault_epoch_processed: Some(InitializeCommandResultRestakingVaultEpochProcessed {
                                            vault: restaking_vault.vault,
                                            supported_token_mint: restaking_vault.supported_token_mint,
                                            items: restaking_vault_epoch_processed_items,
                                        }),
                                    }.into())
                                } else {
                                    let remaining_items =
                                        items.into_iter().skip(1).cloned().collect::<Vec<_>>();
                                    restaking_vault_epoch_process_items = Some(remaining_items);
                                }
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

        Ok((
            result,
            Some({
                if let Some(items) = restaking_vault_epoch_process_items {
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
                                    InitializeCommandState::PrepareRestakingVaultEpochProcess {
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
