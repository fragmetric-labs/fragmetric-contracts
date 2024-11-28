use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};
use crate::errors;
use crate::modules::fund::fund_account_restaking_vault::RestakingVault;
use crate::modules::fund::FundService;
use crate::modules::normalization::{NormalizedTokenPoolAccount, NormalizedTokenPoolService};
use crate::modules::pricing::{PricingService, TokenPricingSource};
use crate::modules::restaking;
use crate::modules::restaking::JitoRestakingVaultService;
use crate::utils::PDASeeds;
use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use jito_bytemuck::AccountDeserialize;
use std::cmp;
use std::ops::Deref;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct RestakeVSTCommand {
    #[max_len(2)]
    items: Vec<RestakeVSTCommandItem>,
    state: RestakeVSTCommandState,
    operation_reserved_restake_token: Option<OperationReservedRestakeToken>,
}

impl From<RestakeVSTCommand> for OperationCommand {
    fn from(command: RestakeVSTCommand) -> Self {
        Self::RestakeVST(command)
    }
}

impl RestakeVSTCommand {
    pub(super) fn new_init(items: Vec<RestakeVSTCommandItem>) -> Self {
        Self {
            items,
            state: RestakeVSTCommandState::Init,
            operation_reserved_restake_token: None,
        }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct RestakeVSTCommandItem {
    vault_address: Pubkey,
    sol_amount: u64,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct RestakeSupportedTokenAsset {
    operation_reserved_amount: u64,
    token_mint: Pubkey,
    token_account: Pubkey,
    token_program: Pubkey,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct OperationReservedRestakeToken {
    operation_reserved_amount: u64,
    token_mint: Pubkey,
}

impl RestakeVSTCommandItem {
    pub(super) fn new(vault_address: Pubkey, sol_amount: u64) -> Self {
        Self {
            vault_address,
            sol_amount,
        }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum RestakeVSTCommandState {
    Init,
    SetupRestake,
    SetupNormalize,
    Normalize(#[max_len(4)] Vec<RestakeSupportedTokenAsset>),
    ReadVaultState,
    Restake,
}

impl SelfExecutable for RestakeVSTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        // let mut vaults = vec![];
        // if let Some(item) = self.items.first() {
        //     vaults.push((
        //         ctx.fund_account.get_restaking_vault_mut(&item.vault_address)?,
        //         item.sol_amount,
        //     ));
        // }
        // let normalized_token_mint = ctx.fund_account.normalized_token.as_ref().unwrap().mint;
        // vaults.sort_by(
        //     |a, b| match (a.0.supported_token_mint, b.0.supported_token_mint) {
        //         (mint, _) if mint == normalized_token_mint => std::cmp::Ordering::Greater,
        //         (_, mint) if mint == normalized_token_mint => std::cmp::Ordering::Less,
        //         _ => std::cmp::Ordering::Equal,
        //     },
        // );
        if let Some(item) = self.items.first() {
            let mut func_account = ctx.fund_account.clone();
            let restaking_vault = func_account.get_restaking_vault_mut(&item.vault_address)?;
            match &self.state {
                RestakeVSTCommandState::Init if item.sol_amount > 0 => {
                    let mut command = self.clone();
                    let normalized_token_mint =
                        &ctx.fund_account.normalized_token.as_ref().unwrap().mint;
                    if &restaking_vault.supported_token_mint == normalized_token_mint {
                        let normalized_token_pool_address =
                            NormalizedTokenPoolService::find_normalized_token_pool_address(
                                &normalized_token_mint,
                            );
                        command.state = RestakeVSTCommandState::SetupNormalize;
                        return Ok(Some(
                            command
                                .with_required_accounts([(normalized_token_pool_address, false)]),
                        ));
                    } else {
                        command.state = RestakeVSTCommandState::SetupRestake;
                        return Ok(Some(command.with_required_accounts([(
                            restaking_vault.supported_token_mint.key(),
                            false,
                        )])));
                    }
                }
                RestakeVSTCommandState::SetupRestake => {
                    let [supported_token_mint, remaining_accounts @ ..] = accounts else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };
                    let mut command = self.clone();
                    command.state = RestakeVSTCommandState::ReadVaultState;
                    let supported_tokens = ctx.fund_account.supported_tokens.clone();
                    for token in supported_tokens {
                        if supported_token_mint.key() != token.mint {
                            continue;
                        }
                        let pricing_service =
                            FundService::new(&mut ctx.receipt_token_mint, &mut ctx.fund_account)?
                                .new_pricing_service(remaining_accounts.to_vec())?;
                        let need_to_restake_token_amount = pricing_service
                            .get_sol_amount_as_token(&token.mint, item.sol_amount)?;
                        let operation_reserved_amount = cmp::min(
                            token.operation_reserved_amount,
                            need_to_restake_token_amount,
                        );
                        command.operation_reserved_restake_token =
                            Some(OperationReservedRestakeToken {
                                token_mint: token.mint,
                                operation_reserved_amount,
                            })
                    }
                    command.state = RestakeVSTCommandState::ReadVaultState;

                    match restaking_vault.receipt_token_pricing_source {
                        TokenPricingSource::JitoRestakingVault { address } => {
                            return Ok(Some(command.with_required_accounts(
                                JitoRestakingVaultService::find_accounts_for_vault(address)?,
                            )));
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    };
                }
                RestakeVSTCommandState::SetupNormalize => {
                    let [normalized_token_pool_address, remaining_accounts @ ..] = accounts else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };
                    let mut command = self.clone();

                    let normalized_token_pool_account =
                        Account::<NormalizedTokenPoolAccount>::try_from(
                            normalized_token_pool_address,
                        )?;

                    let pricing_service =
                        FundService::new(&mut ctx.receipt_token_mint, &mut ctx.fund_account)?
                            .new_pricing_service(remaining_accounts.to_vec())?;
                    let total_reserved_amount_as_sol: u64 = ctx
                        .fund_account
                        .supported_tokens
                        .iter()
                        .filter_map(|t| {
                            normalized_token_pool_account
                                .has_supported_token(&t.mint)
                                .then(|| {
                                    pricing_service
                                        .get_token_amount_as_sol(
                                            &t.mint,
                                            t.operation_reserved_amount,
                                        )
                                        .unwrap()
                                })
                        })
                        .sum();

                    let mut restake_supported_tokens_state = vec![];
                    let supported_tokens = ctx.fund_account.supported_tokens.clone();
                    for token in supported_tokens {
                        if !normalized_token_pool_account.has_supported_token(&token.mint) {
                            continue;
                        }
                        let supported_token_account = ctx
                            .fund_account
                            .find_supported_token_account_address(&token.mint)?;

                        let reserved_token_amount_as_sol = pricing_service
                            .get_token_amount_as_sol(
                                &token.mint,
                                token.operation_reserved_amount,
                            )?;
                        let need_to_restake_token_amount_as_sol_float = (item.sol_amount as f64)
                            * (reserved_token_amount_as_sol as f64)
                            / (total_reserved_amount_as_sol as f64);
                        let need_to_restake_token_amount_as_sol: u64 =
                            need_to_restake_token_amount_as_sol_float as u64;
                        let need_to_restake_token_amount = pricing_service
                            .get_sol_amount_as_token(
                                &token.mint,
                                need_to_restake_token_amount_as_sol,
                            )?;

                        restake_supported_tokens_state.push(RestakeSupportedTokenAsset {
                            token_mint: token.mint,
                            token_account: supported_token_account,
                            token_program: token.program,
                            operation_reserved_amount: cmp::min(
                                token.operation_reserved_amount,
                                need_to_restake_token_amount,
                            ),
                        })
                    }

                    let reserved_normalize_token = restake_supported_tokens_state.first().unwrap();
                    let normalized_token_mint =
                        ctx.fund_account.normalized_token.as_ref().unwrap().mint;
                    let normalized_token_program =
                        ctx.fund_account.normalized_token.as_ref().unwrap().program;
                    let normalized_token_account = ctx
                        .fund_account
                        .find_supported_token_account_address(&normalized_token_mint)?;
                    let pool_supported_token_account =
                        anchor_spl::associated_token::get_associated_token_address(
                            &ctx.fund_account.key(),
                            &reserved_normalize_token.token_mint,
                        );

                    command.state =
                        RestakeVSTCommandState::Normalize(restake_supported_tokens_state.clone());

                    return Ok(Some(command.with_required_accounts([
                        (normalized_token_mint, false),
                        (normalized_token_pool_address.key(), false),
                        (normalized_token_account, false),
                        (normalized_token_program, false),
                        (pool_supported_token_account, false),
                        (reserved_normalize_token.token_mint, false),
                        (reserved_normalize_token.token_account, false),
                        (reserved_normalize_token.token_program, false),
                    ])));
                }
                RestakeVSTCommandState::Normalize(restake_supported_tokens_state) => {
                    let [normalized_token_mint, normalized_token_pool_address, normalized_token_account, normalized_token_program, pool_supported_token_account, supported_token_mint, supported_token_account, supported_token_program, remaining_accounts @ ..] =
                        accounts
                    else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };

                    let mut command = self.clone();
                    if restake_supported_tokens_state.len() == 0 {
                        command.state = RestakeVSTCommandState::ReadVaultState;
                        match restaking_vault.receipt_token_pricing_source {
                            TokenPricingSource::JitoRestakingVault { address } => {
                                return Ok(Some(command.with_required_accounts(
                                    JitoRestakingVaultService::find_accounts_for_vault(address)?,
                                )));
                            }
                            _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                        };
                    }
                    let mut unused_restake_supported_tokens =
                        restake_supported_tokens_state.clone();
                    let token_index = unused_restake_supported_tokens
                        .iter()
                        .position(|t| t.token_mint == supported_token_mint.key())
                        .unwrap();
                    let reserved_restake_token =
                        unused_restake_supported_tokens.swap_remove(token_index);

                    let normalized_token_account_parsed =
                        InterfaceAccount::<TokenAccount>::try_from(normalized_token_account)?;
                    let supported_token_account_parsed =
                        InterfaceAccount::<TokenAccount>::try_from(supported_token_account)?;
                    let pool_supported_token_account_parsed =
                        InterfaceAccount::<TokenAccount>::try_from(pool_supported_token_account)?;
                    let mut normalized_token_mint_parsed =
                        InterfaceAccount::<Mint>::try_from(normalized_token_mint)?;
                    let supported_token_mint_parsed =
                        InterfaceAccount::<Mint>::try_from(supported_token_mint)?;
                    let supported_token_program_parsed =
                        Interface::<TokenInterface>::try_from(*supported_token_program)?;
                    let normalized_token_program_parsed =
                        Program::<Token>::try_from(*normalized_token_program)?;
                    let mut pricing_service =
                        FundService::new(&mut ctx.receipt_token_mint, &mut ctx.fund_account)?
                            .new_pricing_service(remaining_accounts.to_vec())?;
                    let mut normalized_token_pool_account =
                        Account::<NormalizedTokenPoolAccount>::try_from(
                            normalized_token_pool_address,
                        )?;

                    let mut normalized_token_pool_service = NormalizedTokenPoolService::new(
                        &mut normalized_token_pool_account,
                        &mut normalized_token_mint_parsed,
                        &normalized_token_program_parsed,
                    )?;

                    normalized_token_pool_service.normalize_supported_token(
                        &normalized_token_account_parsed,
                        &supported_token_account_parsed,
                        &pool_supported_token_account_parsed,
                        &supported_token_mint_parsed,
                        &supported_token_program_parsed,
                        &ctx.fund_account.as_ref(),
                        &[&ctx.fund_account.get_seeds()],
                        reserved_restake_token.operation_reserved_amount,
                        &mut pricing_service,
                    )?;

                    command.state =
                        RestakeVSTCommandState::Normalize(unused_restake_supported_tokens.clone());
                    let next_reserved_restake_token =
                        unused_restake_supported_tokens.first().unwrap();
                    return Ok(Some(command.with_required_accounts([
                        (normalized_token_mint.key(), false),
                        (normalized_token_pool_address.key(), false),
                        (normalized_token_account.key(), false),
                        (pool_supported_token_account.key(), false),
                        (next_reserved_restake_token.token_mint, false),
                        (next_reserved_restake_token.token_account, false),
                        (next_reserved_restake_token.token_program, false),
                    ])));
                }

                RestakeVSTCommandState::ReadVaultState => {
                    let mut command = self.clone();
                    command.state = RestakeVSTCommandState::Restake;

                    match restaking_vault.receipt_token_pricing_source {
                        TokenPricingSource::JitoRestakingVault { address: _ } => {
                            let [vault_program, vault_account, vault_config, _remaining_accounts @ ..] =
                                accounts
                            else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };
                            return Ok(Some(command.with_required_accounts(
                                JitoRestakingVaultService::find_accounts_for_restaking_vault(
                                    ctx.fund_account.as_ref(),
                                    vault_program,
                                    vault_account,
                                    vault_config,
                                )?,
                            )));
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    }
                }
                RestakeVSTCommandState::Restake => {
                    let mut command = self.clone();

                    match restaking_vault.receipt_token_pricing_source {
                        TokenPricingSource::JitoRestakingVault { address: _ } => {
                            let [jito_vault_program, jito_vault_account, jito_vault_config, vault_update_state_tracker, vault_update_state_tracker_prepare_for_delaying, vault_vrt_mint, vault_vst_mint, fund_supported_token_account, fund_receipt_token_account, vault_fee_wallet_token_account, token_program, system_program, _remaining_accounts @ ..] =
                                accounts
                            else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };

                            let operation_reserved_token =
                                command.operation_reserved_restake_token.unwrap();
                            require_eq!(&operation_reserved_token.token_mint, vault_vst_mint.key);

                            let minted_vrt_amount = JitoRestakingVaultService::new(
                                jito_vault_program.to_account_info(),
                                jito_vault_config.to_account_info(),
                                jito_vault_account.to_account_info(),
                                vault_vrt_mint.to_account_info(),
                                token_program.to_account_info(),
                                token_program.to_account_info(),
                                vault_vst_mint.to_account_info(),
                                fund_supported_token_account.to_account_info(),
                            )?
                            .update_vault_if_needed(
                                ctx.operator,
                                vault_update_state_tracker,
                                vault_update_state_tracker_prepare_for_delaying,
                                Clock::get()?.slot,
                                system_program.as_ref(),
                                &ctx.fund_account.as_ref(),
                                &[&ctx.fund_account.get_seeds().as_ref()],
                            )?
                            .deposit(
                                *fund_supported_token_account,
                                vault_fee_wallet_token_account,
                                *fund_receipt_token_account,
                                operation_reserved_token.operation_reserved_amount,
                                operation_reserved_token.operation_reserved_amount,
                                &ctx.fund_account.as_ref(),
                                &[&ctx.fund_account.get_seeds().as_ref()],
                            )?;

                            restaking_vault.receipt_token_operation_reserved_amount +=
                                minted_vrt_amount;
                            command.operation_reserved_restake_token = None;
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    }
                }
                _ => (),
            }
        }
        if self.items.len() > 1 {
            return Ok(Some(
                RestakeVSTCommand::new_init(self.items[1..].to_vec()).with_required_accounts([]),
            ));
        }

        Ok(None)
    }
}
