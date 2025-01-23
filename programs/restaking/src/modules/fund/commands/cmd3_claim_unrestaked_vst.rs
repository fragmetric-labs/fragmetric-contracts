use anchor_lang::prelude::*;
use anchor_spl::associated_token::spl_associated_token_account;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use jito_bytemuck::AccountDeserialize;
use jito_vault_core::config::Config;
use std::cmp;

use crate::constants::{ADMIN_PUBKEY, JITO_VAULT_PROGRAM_FEE_WALLET};
use crate::errors;
use crate::modules::normalization::{NormalizedTokenPoolAccount, NormalizedTokenPoolService};
use crate::modules::pricing::TokenPricingSource;
use crate::modules::restaking::JitoRestakingVaultService;
use crate::utils::{AccountInfoExt, PDASeeds};

use super::{
    DenormalizeNTCommand, FundService, OperationCommand, OperationCommandContext,
    OperationCommandEntry, OperationCommandResult, RestakeVSTCommand, RestakeVSTCommandState,
    SelfExecutable, UndelegateVSTCommand, WeightedAllocationParticipant,
    WeightedAllocationStrategy,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct ClaimUnrestakedVSTCommand {
    #[max_len(2)]
    items: Vec<ClaimUnrestakedVSTCommandItem>,
    state: ClaimUnrestakedVSTCommandState,
}

impl ClaimUnrestakedVSTCommand {
    pub(super) fn new_init(items: Vec<ClaimUnrestakedVSTCommandItem>) -> Self {
        Self {
            items,
            state: ClaimUnrestakedVSTCommandState::Init,
        }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct ClaimUnrestakedVSTCommandItem {
    vault_address: Pubkey,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct DenormalizeSupportedTokenAsset {
    operation_reserved_amount: u64,
    token_mint: Pubkey,
    token_program: Pubkey,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy)]
pub struct ClaimableUnrestakeWithdrawalTicket {
    withdrawal_ticket_account: Pubkey,
    withdrawal_ticket_token_account: Pubkey,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ClaimableUnrestakeWithdrawalStatus {
    #[max_len(5)]
    withdrawal_tickets: Vec<ClaimableUnrestakeWithdrawalTicket>,
    expected_ncn_epoch: u64,
    delayed_ncn_epoch: u64,
    unrestaked_vst_amount: u64,
}

impl ClaimUnrestakedVSTCommandItem {
    pub(super) fn new(vault_address: Pubkey) -> Self {
        Self { vault_address }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum ClaimUnrestakedVSTCommandState {
    #[default]
    Init,
    Init2,
    ReadVaultState,
    Claim(ClaimableUnrestakeWithdrawalStatus),
    SetupDenormalize(u64),
    Denormalize(#[max_len(4)] Vec<DenormalizeSupportedTokenAsset>),
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ClaimUnrestakedVSTCommandResult {}

impl SelfExecutable for ClaimUnrestakedVSTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        // TODO v0.4.2: ClaimUnrestakedVSTCommand
        return Ok((
            None,
            Some(DenormalizeNTCommand::default().without_required_accounts()),
        ));

        if let Some(item) = self.items.first() {
            match &self.state {
                ClaimUnrestakedVSTCommandState::Init => {
                    let mut command = self.clone();
                    command.state = ClaimUnrestakedVSTCommandState::ReadVaultState;

                    let fund_account = ctx.fund_account.load()?;
                    let restaking_vault = fund_account.get_restaking_vault(&item.vault_address)?;
                    match restaking_vault
                        .receipt_token_pricing_source
                        .try_deserialize()?
                    {
                        Some(TokenPricingSource::JitoRestakingVault { address }) => {
                            let mut required_accounts =
                                JitoRestakingVaultService::find_accounts_to_new(address)?;
                            required_accounts.append(
                                &mut JitoRestakingVaultService::find_withdrawal_tickets(
                                    &restaking_vault.vault,
                                    &ctx.receipt_token_mint.key(),
                                ),
                            );

                            return Ok((
                                None,
                                Some(command.with_required_accounts(required_accounts)),
                            ));
                        }
                        // otherwise fails
                        Some(TokenPricingSource::SPLStakePool { .. })
                        | Some(TokenPricingSource::MarinadeStakePool { .. })
                        | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
                        | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                        | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                        | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                        | None => {
                            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                        #[cfg(all(test, not(feature = "idl-build")))]
                        Some(TokenPricingSource::Mock { .. }) => {
                            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                    };
                }
                ClaimUnrestakedVSTCommandState::ReadVaultState => {
                    let fund_account = ctx.fund_account.load()?;
                    let restaking_vault = fund_account.get_restaking_vault(&item.vault_address)?;
                    match restaking_vault
                        .receipt_token_pricing_source
                        .try_deserialize()?
                    {
                        Some(TokenPricingSource::JitoRestakingVault { address }) => {
                            // require_keys_eq!(address, restaking_vault.vault);
                            //
                            // let [vault_program, vault_config, vault_account, remaining_accounts @ ..] =
                            //     accounts
                            // else {
                            //     err!(ErrorCode::AccountNotEnoughKeys)?
                            // };
                            //
                            // let withdrawal_tickets = &remaining_accounts[0..5];
                            // let _remaining_accounts = &remaining_accounts[5..];
                            //
                            // let (claimable_tickets, _) =
                            //     JitoRestakingVaultService::get_claimable_withdrawal_tickets(
                            //         vault_config,
                            //         &restaking_vault.receipt_token_mint,
                            //         &restaking_vault.receipt_token_program,
                            //         withdrawal_tickets.to_vec(),
                            //     )?;
                            // if claimable_tickets.len() == 0 {
                            //     if self.items.len() > 1 {
                            //         return Ok((
                            //             None,
                            //             Some(
                            //                 ClaimUnrestakedVSTCommand::new_init(
                            //                     self.items[1..].to_vec(),
                            //                 )
                            //                 .with_required_accounts([]),
                            //             ),
                            //         ));
                            //     }
                            //     return Ok((None, None));
                            // };
                            //
                            // let clock = Clock::get()?;
                            // let (vault_update_state_tracker, expected_ncn_epoch) =
                            //     JitoRestakingVaultService::get_vault_update_state_tracker(
                            //         vault_config,
                            //         vault_account,
                            //         clock.slot,
                            //         false,
                            //     )?;
                            //
                            // let (
                            //     vault_update_state_tracker_prepare_for_delaying,
                            //     delayed_ncn_epoch,
                            // ) = JitoRestakingVaultService::get_vault_update_state_tracker(
                            //     vault_config,
                            //     vault_account,
                            //     clock.slot,
                            //     true,
                            // )?;
                            // let mut claimable_unrestaked_tickets = vec![];
                            // for (withdrawal_ticket_account, withdrawal_ticket_token_account) in
                            //     &claimable_tickets
                            // {
                            //     claimable_unrestaked_tickets.push(
                            //         ClaimableUnrestakeWithdrawalTicket {
                            //             withdrawal_ticket_account: *withdrawal_ticket_account,
                            //             withdrawal_ticket_token_account:
                            //                 *withdrawal_ticket_token_account,
                            //         },
                            //     )
                            // }
                            // let mut required_accounts =
                            //     JitoRestakingVaultService::find_accounts_for_unrestaking_vault(
                            //         &ctx.fund_account.to_account_info(),
                            //         vault_program,
                            //         vault_config,
                            //         vault_account,
                            //     )?;
                            //
                            // required_accounts.append(&mut vec![
                            //     (vault_update_state_tracker, true),
                            //     (vault_update_state_tracker_prepare_for_delaying, true),
                            // ]);
                            //
                            // required_accounts.append(&mut vec![
                            //     (claimable_tickets[0].0, true),
                            //     (claimable_tickets[0].1, true),
                            // ]);
                            //
                            // let mut command = self.clone();
                            // command.state = ClaimUnrestakedVSTCommandState::Claim(
                            //     ClaimableUnrestakeWithdrawalStatus {
                            //         withdrawal_tickets: claimable_unrestaked_tickets,
                            //         expected_ncn_epoch,
                            //         delayed_ncn_epoch,
                            //         unrestaked_vst_amount: 0,
                            //     },
                            // );
                            // return Ok((
                            //     None,
                            //     Some(command.with_required_accounts(required_accounts)),
                            // ));
                        }
                        // otherwise fails
                        Some(TokenPricingSource::SPLStakePool { .. })
                        | Some(TokenPricingSource::MarinadeStakePool { .. })
                        | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
                        | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                        | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                        | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                        | None => {
                            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                        #[cfg(all(test, not(feature = "idl-build")))]
                        Some(TokenPricingSource::Mock { .. }) => {
                            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                    };
                }
                ClaimUnrestakedVSTCommandState::Claim(withdrawal_status) => {
                    let mut command = self.clone();

                    let fund_account = ctx.fund_account.load()?;
                    let restaking_vault = fund_account.get_restaking_vault(&item.vault_address)?;
                    match restaking_vault
                        .receipt_token_pricing_source
                        .try_deserialize()?
                    {
                        Some(TokenPricingSource::JitoRestakingVault { .. }) => {
                            let [vault_program, vault_config, vault_account, vault_vrt_mint, vault_vst_mint, fund_supported_token_reserve_account, fund_receipt_token_account, vault_supported_token_account, vault_fee_receipt_token_account, vault_program_fee_wallet_vrt_account, token_program, system_program, vault_update_state_tracker, vault_update_state_tracker_prepare_for_delaying, vault_withdrawal_ticket, vault_withdrawal_ticket_token_account, _remaining_accounts @ ..] =
                                accounts
                            else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };
                            let mut next_withdrawal_status = withdrawal_status.clone();
                            let mut unused_claimable_unrestaked_tickets =
                                next_withdrawal_status.withdrawal_tickets;
                            let token_index = unused_claimable_unrestaked_tickets
                                .iter()
                                .position(|t| {
                                    t.withdrawal_ticket_account == vault_withdrawal_ticket.key()
                                })
                                .unwrap();
                            let _reserved_unrestaked_ticket =
                                unused_claimable_unrestaked_tickets.swap_remove(token_index);

                            // let (current_vault_update_state_tracker, current_epoch, epoch_length) =
                            //     JitoRestakingVaultService::find_current_vault_update_state_tracker(
                            //         &vault_config,
                            //         vault_update_state_tracker,
                            //         withdrawal_status.expected_ncn_epoch,
                            //         vault_update_state_tracker_prepare_for_delaying,
                            //         withdrawal_status.delayed_ncn_epoch,
                            //     )?;

                            // let unrestaked_vst_amount = JitoRestakingVaultService::new(
                            //     vault_program.to_account_info(),
                            //     vault_config.to_account_info(),
                            //     vault_account.to_account_info(),
                            //     vault_vrt_mint.to_account_info(),
                            //     token_program.to_account_info(),
                            //     vault_vst_mint.to_account_info(),
                            //     token_program.to_account_info(),
                            //     vault_supported_token_account.to_account_info(),
                            // )?
                            // .update_vault_if_needed(
                            //     ctx.operator,
                            //     current_vault_update_state_tracker,
                            //     current_epoch,
                            //     epoch_length,
                            //     system_program.as_ref(),
                            //     &ctx.fund_account.to_account_info(),
                            //     &[fund_account.get_seeds().as_ref()],
                            // )?
                            // .withdraw(
                            //     vault_withdrawal_ticket,
                            //     vault_withdrawal_ticket_token_account,
                            //     fund_supported_token_reserve_account,
                            //     vault_fee_receipt_token_account,
                            //     vault_program_fee_wallet_vrt_account,
                            //     &ctx.fund_account.to_account_info(),
                            //     system_program,
                            // )?;

                            // match unused_claimable_unrestaked_tickets.first() {
                            //     Some(next_ticket) => {
                            //         next_withdrawal_status.withdrawal_tickets =
                            //             unused_claimable_unrestaked_tickets.clone();
                            //         next_withdrawal_status.unrestaked_vst_amount +=
                            //             unrestaked_vst_amount;
                            //         command.state = ClaimUnrestakedVSTCommandState::Claim(
                            //             next_withdrawal_status,
                            //         );
                            //
                            //         return Ok((
                            //             None,
                            //             Some(
                            //                 command.with_required_accounts([
                            //                     (vault_program.key(), false),
                            //                     (vault_account.key(), false),
                            //                     (vault_config.key(), false),
                            //                     (vault_vrt_mint.key(), false),
                            //                     (vault_vst_mint.key(), false),
                            //                     (fund_supported_token_reserve_account.key(), false),
                            //                     (fund_receipt_token_account.key(), false),
                            //                     (vault_fee_receipt_token_account.key(), false),
                            //                     (vault_program_fee_wallet_vrt_account.key(), false),
                            //                     (vault_update_state_tracker.key(), false),
                            //                     (
                            //                         vault_update_state_tracker_prepare_for_delaying
                            //                             .key(),
                            //                         false,
                            //                     ),
                            //                     (token_program.key(), false),
                            //                     (system_program.key(), false),
                            //                     (next_ticket.withdrawal_ticket_account, false),
                            //                     (
                            //                         next_ticket.withdrawal_ticket_token_account,
                            //                         false,
                            //                     ),
                            //                 ]),
                            //             ),
                            //         ));
                            //     }
                            //     None => {
                            //         let fund_account = ctx.fund_account.load()?;
                            //         let normalized_token =
                            //             fund_account.get_normalized_token().unwrap();
                            //
                            //         let normalized_token_pool_address =
                            //             NormalizedTokenPoolAccount::find_account_address_by_token_mint(
                            //                 &normalized_token.mint,
                            //             );
                            //
                            //         let normalized_token_account =
                            //             spl_associated_token_account::get_associated_token_address_with_program_id(
                            //                 &ctx.fund_account.key(),
                            //                 &normalized_token.mint,
                            //                 &normalized_token.program,
                            //             );
                            //
                            //         command.state =
                            //             ClaimUnrestakedVSTCommandState::SetupDenormalize(
                            //                 unrestaked_vst_amount,
                            //             );
                            //         return Ok((
                            //             None,
                            //             Some(command.with_required_accounts([
                            //                 (normalized_token.mint, true),
                            //                 (normalized_token_pool_address, true),
                            //                 (normalized_token.program, false),
                            //                 (normalized_token_account, true),
                            //                 (anchor_spl::token::spl_token::native_mint::ID, false), //refactor flag to stop loop => now it always runs a single command in a tx
                            //             ])),
                            //         ));
                            //     }
                            // }
                        }
                        // otherwise fails
                        Some(TokenPricingSource::SPLStakePool { .. })
                        | Some(TokenPricingSource::MarinadeStakePool { .. })
                        | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
                        | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                        | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                        | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                        | None => {
                            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                        #[cfg(all(test, not(feature = "idl-build")))]
                        Some(TokenPricingSource::Mock { .. }) => {
                            err!(errors::ErrorCode::FundOperationCommandExecutionFailedException)?
                        }
                    }
                }
                ClaimUnrestakedVSTCommandState::SetupDenormalize(
                    need_to_denormalize_amount_as_sol,
                ) => {
                    let [normalized_token_mint, normalized_token_pool_address, normalized_token_program, normalized_token_account, _unused_account, remaining_accounts @ ..] =
                        accounts
                    else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };

                    let mut normalized_token_pool_account =
                        Account::<NormalizedTokenPoolAccount>::try_from(
                            normalized_token_pool_address,
                        )?;
                    let mut normalized_token_mint_parsed =
                        InterfaceAccount::<Mint>::try_from(normalized_token_mint)?;
                    let normalized_token_program_parsed =
                        Program::<Token>::try_from(*normalized_token_program)?;

                    let normalized_token_pool_service = NormalizedTokenPoolService::new(
                        &mut normalized_token_pool_account,
                        &mut normalized_token_mint_parsed,
                        &normalized_token_program_parsed,
                    )?;

                    let mut pricing_source = remaining_accounts.to_vec();
                    pricing_source.push(normalized_token_pool_address);
                    let pricing_service =
                        FundService::new(&mut ctx.receipt_token_mint, &mut ctx.fund_account)?
                            .new_pricing_service(pricing_source)?;

                    let mut denormalize_supported_tokens_state =
                        Vec::<DenormalizeSupportedTokenAsset>::new(); // vec![];
                                                                      // for (token_mint, token_program, operation_reserved_amount) in
                                                                      //     normalized_token_pool_service.get_denormalize_tokens_asset(
                                                                      //         &pricing_service,
                                                                      //         *need_to_denormalize_amount_as_sol,
                                                                      //     )?
                                                                      // {
                                                                      //     denormalize_supported_tokens_state.push(DenormalizeSupportedTokenAsset {
                                                                      //         token_mint,
                                                                      //         token_program,
                                                                      //         operation_reserved_amount,
                                                                      //     });
                                                                      // }

                    match denormalize_supported_tokens_state.first() {
                        Some(restake_supported_token_state) => {
                            let pool_supported_token_account =
                                anchor_spl::associated_token::get_associated_token_address(
                                    &normalized_token_pool_address.key(),
                                    &restake_supported_token_state.token_mint,
                                );
                            let reserved_normalize_token_account = ctx
                                .fund_account
                                .load()?
                                .find_supported_token_reserve_account_address(
                                    &restake_supported_token_state.token_mint,
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
                                (pool_supported_token_account, true),
                                (restake_supported_token_state.token_mint, false),
                                (reserved_normalize_token_account, true),
                                (restake_supported_token_state.token_program, false),
                            ];
                            let mut command = self.clone();
                            command.state = ClaimUnrestakedVSTCommandState::Denormalize(
                                denormalize_supported_tokens_state,
                            );
                            return Ok((
                                None,
                                Some(command.with_required_accounts(required_accounts)),
                            ));
                        }
                        None => {
                            if self.items.len() > 1 {
                                return Ok((
                                    None,
                                    Some(
                                        ClaimUnrestakedVSTCommand::new_init(
                                            self.items[1..].to_vec(),
                                        )
                                        .without_required_accounts(),
                                    ),
                                ));
                            }
                            return Ok((None, None));
                        }
                    };
                }
                ClaimUnrestakedVSTCommandState::Denormalize(denormalize_tokens_state) => {
                    let [normalized_token_mint, normalized_token_pool_address, normalized_token_program, normalized_token_account, pool_supported_token_account, supported_token_mint, supported_token_account, supported_token_program, remaining_accounts @ ..] =
                        accounts
                    else {
                        err!(ErrorCode::AccountNotEnoughKeys)?
                    };

                    let mut unused_denormalize_supported_tokens = denormalize_tokens_state.clone();
                    let token_index = unused_denormalize_supported_tokens
                        .iter()
                        .position(|t| t.token_mint == supported_token_mint.key())
                        .unwrap();
                    let reserved_restake_token =
                        unused_denormalize_supported_tokens.swap_remove(token_index);

                    let mut pricing_source = remaining_accounts.to_vec();
                    pricing_source.push(normalized_token_pool_address);
                    let mut pricing_service =
                        FundService::new(&mut ctx.receipt_token_mint, &mut ctx.fund_account)?
                            .new_pricing_service(pricing_source)?;

                    // let mut normalized_token_account_parsed =
                    //     normalized_token_account.parse_interface_account_boxed::<TokenAccount>()?;
                    // let supported_token_account_parsed =
                    //     supported_token_account.parse_interface_account_boxed::<TokenAccount>()?;
                    // let pool_supported_token_account_parsed = pool_supported_token_account
                    //     .parse_interface_account_boxed::<TokenAccount>(
                    // )?;
                    // let mut normalized_token_mint_parsed =
                    //     normalized_token_mint.parse_interface_account_boxed::<Mint>()?;
                    // let supported_token_mint_parsed =
                    //     supported_token_mint.parse_interface_account_boxed::<Mint>()?;
                    // let mut normalized_token_pool_account = normalized_token_pool_address
                    //     .parse_account_boxed::<NormalizedTokenPoolAccount>(
                    // )?;
                    let mut normalized_token_account_parsed =
                        InterfaceAccount::<TokenAccount>::try_from(normalized_token_account)?;
                    let supported_token_account_parsed =
                        InterfaceAccount::<TokenAccount>::try_from(supported_token_account)?;
                    let pool_supported_token_account_parsed =
                        InterfaceAccount::<TokenAccount>::try_from(pool_supported_token_account)?;
                    let mut normalized_token_mint_parsed =
                        InterfaceAccount::<Mint>::try_from(normalized_token_mint)?;
                    let supported_token_mint_parsed =
                        InterfaceAccount::<Mint>::try_from(supported_token_mint)?;
                    let mut normalized_token_pool_account =
                        Account::<NormalizedTokenPoolAccount>::try_from(
                            *normalized_token_pool_address,
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

                    normalized_token_pool_service.denormalize_supported_token(
                        &supported_token_mint_parsed,
                        &supported_token_program_parsed,
                        &pool_supported_token_account_parsed,
                        &mut normalized_token_account_parsed,
                        &supported_token_account_parsed,
                        // signer is fund_reserve_account.
                        &ctx.fund_account.as_ref(),
                        &[ctx.fund_account.load()?.get_seeds().as_ref()],
                        reserved_restake_token.operation_reserved_amount,
                        &mut pricing_service,
                    )?;
                    let mut command = self.clone();
                    return match unused_denormalize_supported_tokens.first() {
                        Some(next_reserved_denormalize_token) => {
                            command.state = ClaimUnrestakedVSTCommandState::Denormalize(
                                unused_denormalize_supported_tokens.clone(),
                            );

                            let next_pool_supported_token_account =
                                anchor_spl::associated_token::get_associated_token_address(
                                    &normalized_token_pool_address.key(),
                                    &next_reserved_denormalize_token.token_mint,
                                );

                            let next_reserved_normalize_token_account = ctx
                                .fund_account
                                .load()?
                                .find_supported_token_reserve_account_address(
                                    &next_reserved_denormalize_token.token_mint,
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
                                (next_reserved_denormalize_token.token_mint, false),
                                (next_reserved_normalize_token_account, true),
                                (next_reserved_denormalize_token.token_program, false),
                            ];
                            return Ok((
                                None,
                                Some(command.with_required_accounts(required_accounts)),
                            ));
                        }
                        _ => Ok((None, None)),
                    };
                }

                _ => (),
            }
        }
        if self.items.len() > 1 {
            return Ok((
                None,
                Some(
                    ClaimUnrestakedVSTCommand::new_init(self.items[1..].to_vec())
                        .with_required_accounts([]),
                ),
            ));
        }
        Ok((None, None))
    }
}
