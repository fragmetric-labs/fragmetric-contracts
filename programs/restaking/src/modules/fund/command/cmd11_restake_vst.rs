use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};
use crate::errors;
use crate::modules::fund::fund_account_restaking_vault::RestakingVault;
use crate::modules::fund::{
    FundService, SupportedToken, WeightedAllocationParticipant, WeightedAllocationStrategy,
    FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
};
use crate::modules::normalization::{NormalizedTokenPoolAccount, NormalizedTokenPoolService};
use crate::modules::pricing::TokenPricingSource;
use crate::modules::restaking;
use crate::modules::restaking::JitoRestakingVaultService;
use crate::utils::{AccountInfoExt, PDASeeds};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::spl_associated_token_account;
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
pub struct NormalizeSupportedTokenAsset {
    operation_reserved_amount: u64,
    token_mint: Pubkey,
    token_program: Pubkey,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct OperationReservedRestakeToken {
    token_mint: Pubkey,
    operation_reserved_amount: u64,
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
    Normalize(#[max_len(4)] Vec<NormalizeSupportedTokenAsset>),
    ReadVaultState,
    Restake([u64; 2]),
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
            match &self.state {
                RestakeVSTCommandState::Init if item.sol_amount > 0 => {
                    let mut command = self.clone();

                    let fund_account = ctx.fund_account.load()?;
                    let restaking_vault = fund_account.get_restaking_vault(&item.vault_address)?;
                    if let Some(normalized_token) = fund_account.get_normalized_token() {
                        if normalized_token.mint == restaking_vault.supported_token_mint {
                            let normalized_token_pool_address =
                                NormalizedTokenPoolAccount::find_account_address_by_token_mint(
                                    &normalized_token.mint,
                                );

                            let normalized_token_account =
                                spl_associated_token_account::get_associated_token_address_with_program_id(
                                    &ctx.fund_account.key(),
                                    &normalized_token.mint,
                                    &normalized_token.program,
                                );
                            command.state = RestakeVSTCommandState::SetupNormalize;
                            return Ok(Some(command.with_required_accounts([
                                (normalized_token.mint, true),
                                (normalized_token_pool_address, true),
                                (normalized_token.program, false),
                                (normalized_token_account, true),
                            ])));
                        }
                    }

                    command.state = RestakeVSTCommandState::SetupRestake;
                    return Ok(Some(command.with_required_accounts([(
                        restaking_vault.supported_token_mint.key(),
                        false,
                    )])));
                }
                RestakeVSTCommandState::SetupRestake => {
                    let [supported_token_mint, remaining_accounts @ ..] = accounts else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };

                    let mut command = self.clone();
                    command.state = RestakeVSTCommandState::ReadVaultState;

                    let (supported_token_mint, operation_reserved_amount) = {
                        let (supported_token_mint, operation_reserved_amount) = match ctx
                            .fund_account
                            .load()?
                            .get_supported_tokens_iter()
                            .find(|t| t.mint == supported_token_mint.key())
                        {
                            Some(supported_token) => (
                                supported_token.mint,
                                supported_token.token.operation_reserved_amount,
                            ),
                            None => err!(errors::ErrorCode::FundNotSupportedTokenError)?,
                        };

                        let pricing_service =
                            FundService::new(&mut ctx.receipt_token_mint, &mut ctx.fund_account)?
                                .new_pricing_service(remaining_accounts.to_vec())?;
                        let need_to_restake_token_amount = pricing_service
                            .get_sol_amount_as_token(&supported_token_mint, item.sol_amount)?;

                        (
                            supported_token_mint,
                            cmp::min(operation_reserved_amount, need_to_restake_token_amount),
                        )
                    };

                    let fund_account = ctx.fund_account.load()?;
                    let restaking_vault = fund_account.get_restaking_vault(&item.vault_address)?;
                    let supported_token = match fund_account
                        .get_supported_tokens_iter()
                        .find(|t| t.mint == supported_token_mint.key())
                    {
                        Some(supported_token) => supported_token,
                        None => err!(errors::ErrorCode::FundNotSupportedTokenError)?,
                    };

                    command.operation_reserved_restake_token =
                        Some(OperationReservedRestakeToken {
                            token_mint: supported_token.mint,
                            operation_reserved_amount,
                        });

                    match restaking_vault
                        .receipt_token_pricing_source
                        .try_deserialize()?
                    {
                        Some(TokenPricingSource::JitoRestakingVault { address }) => {
                            return Ok(Some(command.with_required_accounts(
                                JitoRestakingVaultService::find_accounts_for_vault(address)?,
                            )));
                        }
                        _ => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
                    };
                }
                RestakeVSTCommandState::SetupNormalize => {
                    let [normalized_token_mint, normalized_token_pool_address, normalized_token_program, normalized_token_account, remaining_accounts @ ..] =
                        accounts
                    else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };
                    let mut command = self.clone();

                    let normalized_token_pool_account =
                        Account::<NormalizedTokenPoolAccount>::try_from(
                            normalized_token_pool_address,
                        )?;

                    let mut pricing_source = remaining_accounts.to_vec();
                    pricing_source.push(normalized_token_pool_address);

                    let pricing_service =
                        FundService::new(&mut ctx.receipt_token_mint, &mut ctx.fund_account)?
                            .new_pricing_service(pricing_source)?;

                    let mut participants = vec![];
                    let fund_account = ctx.fund_account.load()?;
                    let restaking_vault = fund_account.get_restaking_vault(&item.vault_address)?;
                    let supported_tokens = fund_account
                        .get_supported_tokens_iter()
                        .filter_map(|t| {
                            if t.token.operation_reserved_amount == 0 {
                                None
                            } else {
                                if normalized_token_pool_account.has_supported_token(&t.mint)
                                    && t.mint != normalized_token_mint.key()
                                {
                                    let reserved_amount_as_sol = pricing_service
                                        .get_token_amount_as_sol(
                                            &t.mint,
                                            t.token.operation_reserved_amount,
                                        )
                                        .unwrap();
                                    participants.push(WeightedAllocationParticipant::new(
                                        reserved_amount_as_sol,
                                        0,
                                        u64::MAX,
                                    ));
                                    Some(t)
                                } else {
                                    None
                                }
                            }
                        })
                        .collect::<Vec<_>>();

                    let mut strategy = WeightedAllocationStrategy::<
                        FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
                    >::new(participants);
                    let _ = strategy.put(item.sol_amount);

                    let mut restake_supported_tokens_state = vec![];
                    for (i, supported_token) in supported_tokens.iter().enumerate() {
                        let need_to_restake_token_amount = pricing_service
                            .get_sol_amount_as_token(
                                &supported_token.mint,
                                strategy.get_participant_last_put_amount_by_index(i)?,
                            )?;

                        restake_supported_tokens_state.push(NormalizeSupportedTokenAsset {
                            token_mint: supported_token.mint,
                            token_program: supported_token.program,
                            operation_reserved_amount: cmp::min(
                                supported_token.token.operation_reserved_amount,
                                need_to_restake_token_amount,
                            ),
                        });
                    }
                    return match restake_supported_tokens_state.first() {
                        Some(restake_supported_token_state) => {
                            let pool_supported_token_account =
                                anchor_spl::associated_token::get_associated_token_address(
                                    &normalized_token_pool_address.key(),
                                    &restake_supported_token_state.token_mint,
                                );
                            let reserved_normalize_token_account = fund_account
                                .find_supported_token_account_address(
                                    &restake_supported_token_state.token_mint,
                                )?;

                            command.state = RestakeVSTCommandState::Normalize(
                                restake_supported_tokens_state.clone(),
                            );
                            let required_accounts = vec![
                                (
                                    normalized_token_mint.key(),
                                    normalized_token_mint.is_writable,
                                ),
                                (
                                    normalized_token_pool_address.key(),
                                    normalized_token_pool_address.is_writable,
                                ),
                                (
                                    normalized_token_program.key(),
                                    normalized_token_program.is_writable,
                                ),
                                (
                                    normalized_token_account.key(),
                                    normalized_token_account.is_writable,
                                ),
                                (pool_supported_token_account, true),
                                (restake_supported_token_state.token_mint, false),
                                (reserved_normalize_token_account, true),
                                (restake_supported_token_state.token_program, false),
                            ];

                            Ok(Some(command.with_required_accounts(required_accounts)))
                        }
                        None => {
                            if self.items.len() > 1 {
                                return Ok(Some(
                                    RestakeVSTCommand::new_init(self.items[1..].to_vec())
                                        .without_required_accounts(),
                                ));
                            }
                            Ok(None)
                        }
                    };
                }
                RestakeVSTCommandState::Normalize(restake_supported_tokens_state) => {
                    let [normalized_token_mint, normalized_token_pool_address, normalized_token_program, normalized_token_account, pool_supported_token_account, supported_token_mint, supported_token_account, supported_token_program, remaining_accounts @ ..] =
                        accounts
                    else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };

                    let mut unused_restake_supported_tokens =
                        restake_supported_tokens_state.clone();
                    let token_index = unused_restake_supported_tokens
                        .iter()
                        .position(|t| t.token_mint == supported_token_mint.key())
                        .unwrap();
                    let reserved_restake_token =
                        unused_restake_supported_tokens.swap_remove(token_index);

                    let mut pricing_source = remaining_accounts.to_vec();
                    pricing_source.push(normalized_token_pool_address);
                    let mut pricing_service =
                        FundService::new(&mut ctx.receipt_token_mint, &mut ctx.fund_account)?
                            .new_pricing_service(pricing_source)?;

                    let fund_account = ctx.fund_account.load()?;
                    let restaking_vault = fund_account.get_restaking_vault(&item.vault_address)?;

                    let mut normalized_token_account_parsed =
                        normalized_token_account.parse_interface_account_boxed::<TokenAccount>()?;
                    let supported_token_account_parsed =
                        supported_token_account.parse_interface_account_boxed::<TokenAccount>()?;
                    let pool_supported_token_account_parsed = pool_supported_token_account
                        .parse_interface_account_boxed::<TokenAccount>(
                    )?;
                    let mut normalized_token_mint_parsed =
                        normalized_token_mint.parse_interface_account_boxed::<Mint>()?;
                    let supported_token_mint_parsed =
                        supported_token_mint.parse_interface_account_boxed::<Mint>()?;
                    let mut normalized_token_pool_account = normalized_token_pool_address
                        .parse_account_boxed::<NormalizedTokenPoolAccount>(
                    )?;
                    let supported_token_program_parsed =
                        Interface::<TokenInterface>::try_from(*supported_token_program)?;
                    let normalized_token_program_parsed =
                        Program::<Token>::try_from(*normalized_token_program)?;

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
                        &ctx.fund_account.to_account_info(),
                        &[fund_account.get_seeds().as_ref()],
                        reserved_restake_token.operation_reserved_amount,
                        &mut pricing_service,
                    )?;

                    let mut command = self.clone();
                    match unused_restake_supported_tokens.first() {
                        Some(next_reserved_restake_token) => {
                            command.state = RestakeVSTCommandState::Normalize(
                                unused_restake_supported_tokens.clone(),
                            );

                            let next_pool_supported_token_account =
                                anchor_spl::associated_token::get_associated_token_address(
                                    &normalized_token_pool_address.key(),
                                    &next_reserved_restake_token.token_mint,
                                );

                            let next_reserved_normalize_token_account = fund_account
                                .find_supported_token_account_address(
                                    &next_reserved_restake_token.token_mint,
                                )?;

                            let required_accounts = vec![
                                (
                                    normalized_token_mint.key(),
                                    normalized_token_mint.is_writable,
                                ),
                                (
                                    normalized_token_pool_address.key(),
                                    normalized_token_pool_address.is_writable,
                                ),
                                (
                                    normalized_token_program.key(),
                                    normalized_token_program.is_writable,
                                ),
                                (
                                    normalized_token_account.key(),
                                    normalized_token_account.is_writable,
                                ),
                                (next_pool_supported_token_account, true),
                                (next_reserved_restake_token.token_mint, false),
                                (next_reserved_normalize_token_account, true),
                                (next_reserved_restake_token.token_program, false),
                            ];
                            return Ok(Some(command.with_required_accounts(required_accounts)));
                        }
                        None => {
                            command.state = RestakeVSTCommandState::ReadVaultState;
                            match restaking_vault
                                .receipt_token_pricing_source
                                .try_deserialize()?
                            {
                                Some(TokenPricingSource::JitoRestakingVault { address }) => {
                                    normalized_token_account_parsed.reload()?;
                                    command.operation_reserved_restake_token =
                                        Some(OperationReservedRestakeToken {
                                            token_mint: normalized_token_account_parsed.mint,
                                            operation_reserved_amount:
                                                normalized_token_account_parsed.amount,
                                        });
                                    return Ok(Some(command.with_required_accounts(
                                        JitoRestakingVaultService::find_accounts_for_vault(
                                            address,
                                        )?,
                                    )));
                                }
                                _ => err!(
                                    errors::ErrorCode::FundOperationCommandExecutionFailedException
                                )?,
                            };
                        }
                    }
                }

                RestakeVSTCommandState::ReadVaultState => {
                    let mut command = self.clone();

                    let fund_account = ctx.fund_account.load()?;
                    let restaking_vault = fund_account.get_restaking_vault(&item.vault_address)?;

                    match restaking_vault
                        .receipt_token_pricing_source
                        .try_deserialize()?
                    {
                        Some(TokenPricingSource::JitoRestakingVault { .. }) => {
                            let [vault_program, vault_account, vault_config, _remaining_accounts @ ..] =
                                accounts
                            else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };

                            let mut required_accounts =
                                JitoRestakingVaultService::find_accounts_for_restaking_vault(
                                    ctx.fund_account.as_ref(),
                                    vault_program,
                                    vault_config,
                                    vault_account,
                                )?;

                            let clock = Clock::get()?;
                            let (vault_update_state_tracker, expected_ncn_epoch) =
                                JitoRestakingVaultService::get_vault_update_state_tracker(
                                    vault_config,
                                    clock.slot,
                                    false,
                                )?;
                            let (
                                vault_update_state_tracker_prepare_for_delaying,
                                delayed_ncn_epoch,
                            ) = JitoRestakingVaultService::get_vault_update_state_tracker(
                                vault_config,
                                clock.slot,
                                true,
                            )?;

                            required_accounts.append(&mut vec![
                                (vault_update_state_tracker, true),
                                (vault_update_state_tracker_prepare_for_delaying, true),
                            ]);

                            command.state = RestakeVSTCommandState::Restake([
                                expected_ncn_epoch,
                                delayed_ncn_epoch,
                            ]);
                            return Ok(Some(command.with_required_accounts(required_accounts)));
                        }
                        _ => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
                    }
                }
                RestakeVSTCommandState::Restake(ncn_epoch) => {
                    let mut command = self.clone();

                    let fund_account = ctx.fund_account.load()?;
                    let restaking_vault = fund_account.get_restaking_vault(&item.vault_address)?;

                    match restaking_vault
                        .receipt_token_pricing_source
                        .try_deserialize()?
                    {
                        Some(TokenPricingSource::JitoRestakingVault { .. }) => {
                            let [jito_vault_program, jito_vault_config, jito_vault_account, vault_vrt_mint, vault_vst_mint, fund_supported_token_account, fund_receipt_token_account, vault_supported_token_account, vault_fee_wallet_token_account, token_program, system_program, vault_update_state_tracker, vault_update_state_tracker_prepare_for_delaying, _remaining_accounts @ ..] =
                                accounts
                            else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };

                            let operation_reserved_token =
                                command.operation_reserved_restake_token.unwrap();
                            require_eq!(&operation_reserved_token.token_mint, vault_vst_mint.key);

                            let (current_vault_update_state_tracker, current_epoch, epoch_length) =
                                JitoRestakingVaultService::find_current_vault_update_state_tracker(
                                    &jito_vault_config,
                                    vault_update_state_tracker,
                                    ncn_epoch[0],
                                    vault_update_state_tracker_prepare_for_delaying,
                                    ncn_epoch[1],
                                )?;

                            let minted_vrt_amount = JitoRestakingVaultService::new(
                                jito_vault_program.to_account_info(),
                                jito_vault_config.to_account_info(),
                                jito_vault_account.to_account_info(),
                                vault_vrt_mint.to_account_info(),
                                token_program.to_account_info(),
                                token_program.to_account_info(),
                                vault_vst_mint.to_account_info(),
                                vault_supported_token_account.to_account_info(),
                            )?
                            .update_vault_if_needed(
                                ctx.operator,
                                current_vault_update_state_tracker,
                                current_epoch,
                                epoch_length,
                                system_program.as_ref(),
                                &ctx.fund_account.to_account_info(),
                                &[fund_account.get_seeds().as_ref()],
                            )?
                            .deposit(
                                fund_supported_token_account,
                                vault_fee_wallet_token_account,
                                fund_receipt_token_account,
                                operation_reserved_token.operation_reserved_amount,
                                operation_reserved_token.operation_reserved_amount,
                                &ctx.fund_account.to_account_info(),
                                &[fund_account.get_seeds().as_ref()],
                            )?;
                            {
                                let mut fund_account = ctx.fund_account.load_mut()?;
                                let mut restaking_vault =
                                    fund_account.get_restaking_vault_mut(&item.vault_address)?;
                                restaking_vault.receipt_token_operation_reserved_amount +=
                                    minted_vrt_amount;
                            }
                            command.operation_reserved_restake_token = None;
                        }
                        _ => err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?,
                    }
                }
                _ => (),
            }
        }

        if self.items.len() > 1 {
            return Ok(Some(
                RestakeVSTCommand::new_init(self.items[1..].to_vec()).without_required_accounts(),
            ));
        }

        Ok(None)
    }
}
