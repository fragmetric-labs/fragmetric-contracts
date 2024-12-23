use anchor_lang::prelude::*;
use anchor_spl::associated_token::spl_associated_token_account;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use std::cmp;
use std::ops::Deref;

use crate::errors;
use crate::modules::normalization::{NormalizedTokenPoolAccount, NormalizedTokenPoolService};
use crate::modules::pricing::TokenPricingSource;
use crate::modules::restaking::JitoRestakingVaultService;
use crate::utils::{AccountInfoExt, PDASeeds};

use super::{
    FundService, HarvestRewardCommand, OperationCommand, OperationCommandContext,
    OperationCommandEntry, OperationCommandResult, RestakingVault, SelfExecutable, SupportedToken,
    WeightedAllocationParticipant, WeightedAllocationStrategy, FUND_ACCOUNT_MAX_RESTAKING_VAULTS,
    FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct RestakeVSTCommand {
    state: RestakeVSTCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct RestakeVSTCommandItem {
    vault: Pubkey,
    supported_token_mint: Pubkey,
    allocated_token_amount: u64,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum RestakeVSTCommandState {
    /// Initializes a command with items based on the fund state and strategy.
    #[default]
    New,
    /// Prepares to execute restaking for the first item in the list.
    Prepare {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        items: Vec<RestakeVSTCommandItem>,
    },
    /// Executes restaking for the first item and transitions to the next command,
    /// either preparing the next item or performing a delegation operation.
    Execute {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        items: Vec<RestakeVSTCommandItem>,
    },
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct RestakeVSTCommandResult {}

impl SelfExecutable for RestakeVSTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let mut remaining_items: Option<Vec<RestakeVSTCommandItem>> = None;
        let mut result: Option<OperationCommandResult> = None;

        match &self.state {
            RestakeVSTCommandState::New => {
                let pricing_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                    .new_pricing_service(accounts.into_iter().cloned())?;
                let fund_account = ctx.fund_account.load()?;

                // find restakable tokens with their restakable amount among ST and NT
                let restaking_vaults = fund_account.get_restaking_vaults_iter().collect::<Vec<_>>();
                let mut restakable_token_and_amounts = fund_account
                    .get_supported_tokens_iter()
                    .map(|supported_token| {
                        let supported_token_net_operation_reserved_amount = fund_account
                            .get_asset_net_operation_reserved_amount(
                                Some(supported_token.mint),
                                &pricing_service,
                            )?;
                        Ok((
                            supported_token.mint,
                            if supported_token_net_operation_reserved_amount > 0 {
                                let supported_token_restaking_reserved_amount =
                                    u64::try_from(supported_token_net_operation_reserved_amount)?;
                                supported_token_restaking_reserved_amount
                            } else {
                                0
                            },
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?;
                if let Some(normalized_token) = fund_account.get_normalized_token() {
                    restakable_token_and_amounts.push((
                        normalized_token.mint,
                        normalized_token.operation_reserved_amount,
                    ))
                }

                // calculate allocation of tokens for each restaking vaults
                let mut items =
                    Vec::<RestakeVSTCommandItem>::with_capacity(FUND_ACCOUNT_MAX_RESTAKING_VAULTS);
                for (token_mint, token_amount) in restakable_token_and_amounts {
                    let restakable_vaults = fund_account
                        .get_restaking_vaults_iter()
                        .filter(|restaking_vault| {
                            restaking_vault.supported_token_mint == token_mint
                        })
                        .collect::<Vec<_>>();
                    if restakable_vaults.is_empty() {
                        continue;
                    }

                    let mut strategy =
                        WeightedAllocationStrategy::<FUND_ACCOUNT_MAX_RESTAKING_VAULTS>::new(
                            restakable_vaults
                                .iter()
                                .map(|restaking_vault| {
                                    Ok(WeightedAllocationParticipant::new(
                                        restaking_vault.sol_allocation_weight,
                                        pricing_service.get_token_amount_as_sol(
                                            &restaking_vault.receipt_token_mint,
                                            restaking_vault.receipt_token_operation_reserved_amount,
                                        )?,
                                        restaking_vault.sol_allocation_capacity_amount,
                                    ))
                                })
                                .collect::<Result<Vec<_>>>()?,
                        );

                    strategy
                        .put(pricing_service.get_token_amount_as_sol(&token_mint, token_amount)?)?;

                    for (index, strategy_participant) in
                        strategy.get_participants_iter().enumerate()
                    {
                        let allocated_token_amount = strategy_participant.get_last_put_amount()?;
                        if allocated_token_amount > 0 {
                            let restaking_vault = restakable_vaults.get(index).unwrap();
                            match items
                                .iter_mut()
                                .find(|item| item.vault == restaking_vault.vault)
                            {
                                None => items.push(RestakeVSTCommandItem {
                                    vault: restaking_vault.vault,
                                    supported_token_mint: restaking_vault.supported_token_mint,
                                    allocated_token_amount,
                                }),
                                Some(item) => item.allocated_token_amount += allocated_token_amount,
                            };
                        }
                    }
                }

                if items.len() > 0 {
                    remaining_items = Some(items);
                }
            }
            // RestakeVSTCommandState::ReadVaultState => {
            //     let mut command = self.clone();
            //
            //     let fund_account = ctx.fund_account.load()?;
            //     let restaking_vault = fund_account.get_restaking_vault(&item.vault_address)?;
            //
            //     match restaking_vault
            //         .receipt_token_pricing_source
            //         .try_deserialize()?
            //     {
            //         Some(TokenPricingSource::JitoRestakingVault { .. }) => {
            //             let [vault_program, vault_account, vault_config, _remaining_accounts @ ..] =
            //                 accounts
            //             else {
            //                 err!(ErrorCode::AccountNotEnoughKeys)?
            //             };
            //
            //             let mut required_accounts =
            //                 JitoRestakingVaultService::find_accounts_for_restaking_vault(
            //                     ctx.fund_account.as_ref(),
            //                     vault_program,
            //                     vault_config,
            //                     vault_account,
            //                 )?;
            //
            //             let clock = Clock::get()?;
            //             let (vault_update_state_tracker, expected_ncn_epoch) =
            //                 JitoRestakingVaultService::get_vault_update_state_tracker(
            //                     vault_config,
            //                     vault_account,
            //                     clock.slot,
            //                     false,
            //                 )?;
            //             let (
            //                 vault_update_state_tracker_prepare_for_delaying,
            //                 delayed_ncn_epoch,
            //             ) = JitoRestakingVaultService::get_vault_update_state_tracker(
            //                 vault_config,
            //                 vault_account,
            //                 clock.slot,
            //                 true,
            //             )?;
            //
            //             required_accounts.append(&mut vec![
            //                 (vault_update_state_tracker, true),
            //                 (vault_update_state_tracker_prepare_for_delaying, true),
            //             ]);
            //
            //             command.state = RestakeVSTCommandState::Restake([
            //                 expected_ncn_epoch,
            //                 delayed_ncn_epoch,
            //             ]);
            //             return Ok((
            //                 None,
            //                 Some(command.with_required_accounts(required_accounts)),
            //             ));
            //         }
            //         _ => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
            //     }
            // }
            // RestakeVSTCommandState::Restake(ncn_epoch) => {
            //     let mut command = self.clone();
            //
            //     let fund_account = ctx.fund_account.load()?;
            //     let restaking_vault = fund_account.get_restaking_vault(&item.vault_address)?;
            //
            //     match restaking_vault
            //         .receipt_token_pricing_source
            //         .try_deserialize()?
            //     {
            //         Some(TokenPricingSource::JitoRestakingVault { .. }) => {
            //             let [jito_vault_program, jito_vault_account, jito_vault_config, vault_update_state_tracker, vault_update_state_tracker_prepare_for_delaying, vault_vrt_mint, vault_vst_mint, fund_supported_token_reserve_account, fund_receipt_token_account, vault_supported_token_account, vault_fee_wallet_token_account, token_program, system_program, _remaining_accounts @ ..] =
            //                 accounts
            //             else {
            //                 err!(ErrorCode::AccountNotEnoughKeys)?
            //             };
            //
            //             let operation_reserved_token =
            //                 command.operation_reserved_restake_token.unwrap();
            //             require_eq!(&operation_reserved_token.token_mint, vault_vst_mint.key);
            //
            //             let (current_vault_update_state_tracker, current_epoch, epoch_length) =
            //                 JitoRestakingVaultService::find_current_vault_update_state_tracker(
            //                     &jito_vault_config,
            //                     vault_update_state_tracker,
            //                     ncn_epoch[0],
            //                     vault_update_state_tracker_prepare_for_delaying,
            //                     ncn_epoch[1],
            //                 )?;
            //
            //             let minted_vrt_amount = JitoRestakingVaultService::new(
            //                 jito_vault_program.to_account_info(),
            //                 jito_vault_config.to_account_info(),
            //                 jito_vault_account.to_account_info(),
            //                 vault_vrt_mint.to_account_info(),
            //                 token_program.to_account_info(),
            //                 token_program.to_account_info(),
            //                 vault_vst_mint.to_account_info(),
            //                 vault_supported_token_account.to_account_info(),
            //             )?
            //                 .update_vault_if_needed(
            //                     ctx.operator,
            //                     current_vault_update_state_tracker,
            //                     current_epoch,
            //                     epoch_length,
            //                     system_program.as_ref(),
            //                     &ctx.fund_account.to_account_info(),
            //                     &[fund_account.get_seeds().as_ref()],
            //                 )?
            //                 .deposit(
            //                     *fund_supported_token_reserve_account,
            //                     vault_fee_wallet_token_account,
            //                     *fund_receipt_token_account,
            //                     operation_reserved_token.operation_reserved_amount,
            //                     operation_reserved_token.operation_reserved_amount,
            //                     &ctx.fund_account.to_account_info(),
            //                     &[fund_account.get_seeds().as_ref()],
            //                 )?;
            //             {
            //                 let mut fund_account = ctx.fund_account.load_mut()?;
            //                 let restaking_vault =
            //                     fund_account.get_restaking_vault_mut(&item.vault_address)?;
            //                 restaking_vault.receipt_token_operation_reserved_amount +=
            //                     minted_vrt_amount;
            //             }
            //             command.operation_reserved_restake_token = None;
            //         }
            //         _ => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
            //     }
            // }
            _ => {}
        }

        // transition to next command
        Ok((
            result,
            match remaining_items {
                Some(remaining_items) if remaining_items.len() > 0 => {
                    let pricing_source = ctx
                        .fund_account
                        .load()?
                        .get_restaking_vault(&remaining_items.first().unwrap().vault)?
                        .receipt_token_pricing_source
                        .try_deserialize()?
                        .ok_or_else(|| {
                            error!(errors::ErrorCode::FundOperationCommandExecutionFailedException)
                        })?;

                    Some(
                        RestakeVSTCommand {
                            state: RestakeVSTCommandState::Prepare {
                                items: remaining_items,
                            },
                        }
                        .with_required_accounts(match pricing_source {
                            TokenPricingSource::JitoRestakingVault { address } => {
                                JitoRestakingVaultService::find_accounts_for_vault(address)?
                            }
                            _ => err!(
                                errors::ErrorCode::FundOperationCommandExecutionFailedException
                            )?,
                        }),
                    )
                }
                _ => Some(HarvestRewardCommand::default().without_required_accounts()),
            },
        ))
    }
}
