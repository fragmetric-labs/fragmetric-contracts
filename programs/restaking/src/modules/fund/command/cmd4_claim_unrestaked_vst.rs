use std::cmp;
use super::{NormalizeSupportedTokenAsset, OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};
use crate::constants::{ADMIN_PUBKEY, JITO_VAULT_PROGRAM_FEE_WALLET};
use crate::errors;
use crate::modules::normalization::{NormalizedTokenPoolAccount, NormalizedTokenPoolService};
use crate::modules::pricing::TokenPricingSource;
use crate::modules::restaking::JitoRestakingVaultService;
use crate::utils::PDASeeds;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::spl_associated_token_account;
use crate::modules::fund::FundService;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ClaimUnrestakedVSTCommand {
    #[max_len(2)]
    items: Vec<ClaimUnrestakedVSTCommandItem>,
    state: ClaimUnrestakedVSTCommandState,
}

impl From<ClaimUnrestakedVSTCommand> for OperationCommand {
    fn from(command: ClaimUnrestakedVSTCommand) -> Self {
        Self::ClaimUnrestakedVST(command)
    }
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
pub struct ClaimableUnrestakeWithdrawalTicket {
    withdrawal_ticket_account: Pubkey,
    withdrawal_ticket_token_account: Pubkey,
}

impl ClaimUnrestakedVSTCommandItem {
    pub(super) fn new(vault_address: Pubkey) -> Self {
        Self { vault_address }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum ClaimUnrestakedVSTCommandState {
    Init,
    ReadVaultState,
    Claim(#[max_len(5)] Vec<ClaimableUnrestakeWithdrawalTicket>),
    SetupDenormalize(u64),
}

impl SelfExecutable for ClaimUnrestakedVSTCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        if let Some(item) = self.items.first() {
            let mut func_account = ctx.fund_account.clone();
            let restaking_vault = func_account.get_restaking_vault_mut(&item.vault_address)?;

            match &self.state {
                ClaimUnrestakedVSTCommandState::Init => {
                    let mut command = self.clone();
                    command.state = ClaimUnrestakedVSTCommandState::ReadVaultState;
                    match restaking_vault.receipt_token_pricing_source {
                        TokenPricingSource::JitoRestakingVault { address } => {
                            let mut required_accounts =
                                JitoRestakingVaultService::find_accounts_for_vault(address)?;
                            required_accounts
                                .append(&mut JitoRestakingVaultService::find_withdrawal_tickets());
                            return Ok(Some(command.with_required_accounts(required_accounts)));
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    };
                }
                ClaimUnrestakedVSTCommandState::ReadVaultState => {
                    match restaking_vault.receipt_token_pricing_source {
                        TokenPricingSource::JitoRestakingVault { address } => {
                            let [vault_program, vault_account, vault_config, remaining_accounts @ ..] =
                                accounts
                            else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };

                            let withdrawal_tickets = &remaining_accounts[0..5] else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };
                            let remaining_accounts = &remaining_accounts[5..] else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };

                            let (claimable_tickets, _) =
                                JitoRestakingVaultService::get_claimable_withdrawal_tickets(
                                    vault_config,
                                    withdrawal_tickets.to_vec(),
                                )?;

                            if claimable_tickets.len() == 0 {
                                if self.items.len() > 1 {
                                    return Ok(Some(
                                        ClaimUnrestakedVSTCommand::new_init(
                                            self.items[1..].to_vec(),
                                        )
                                        .with_required_accounts([]),
                                    ));
                                }
                                return Ok(None);
                            };

                            let mut required_accounts =
                                JitoRestakingVaultService::find_accounts_for_unrestaking_vault(
                                    &ctx.fund_account.to_account_info(),
                                    vault_program,
                                    vault_account,
                                    vault_config,
                                )?;

                            let mut claimable_unrestaked_tickets = vec![];
                            for (withdrawal_ticket_account, withdrawal_ticket_token_account) in
                                &claimable_tickets
                            {
                                claimable_unrestaked_tickets.push(
                                    ClaimableUnrestakeWithdrawalTicket {
                                        withdrawal_ticket_account: *withdrawal_ticket_account,
                                        withdrawal_ticket_token_account:
                                            *withdrawal_ticket_token_account,
                                    },
                                )
                            }

                            required_accounts.append(&mut vec![
                                (claimable_tickets[0].0, false),
                                (claimable_tickets[0].1, false),
                            ]);

                            let mut command = self.clone();
                            command.state =
                                ClaimUnrestakedVSTCommandState::Claim(claimable_unrestaked_tickets);
                            return Ok(Some(command.with_required_accounts(required_accounts)));
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    };
                }
                ClaimUnrestakedVSTCommandState::Claim(claimable_unrestaked_tickets) => {
                    let mut command = self.clone();
                    match restaking_vault.receipt_token_pricing_source {
                        TokenPricingSource::JitoRestakingVault { address: _ } => {
                            let [vault_program, vault_account, vault_config, vault_vrt_mint, vault_vst_mint, fund_supported_token_account, fund_receipt_token_account, vault_fee_receipt_token_account, vault_program_fee_wallet_vrt_account, vault_update_state_tracker, vault_update_state_tracker_prepare_for_delaying, token_program, system_program, vault_withdrawal_ticket, vault_withdrawal_ticket_token_account, remaining_accounts @ ..] =
                                accounts
                            else {
                                err!(ErrorCode::AccountNotEnoughKeys)?
                            };

                            let mut unused_claimable_unrestaked_tickets =
                                claimable_unrestaked_tickets.clone();
                            let token_index = unused_claimable_unrestaked_tickets
                                .iter()
                                .position(|t| {
                                    t.withdrawal_ticket_account == vault_withdrawal_ticket.key()
                                })
                                .unwrap();
                            let reserved_unrestaked_ticket =
                                unused_claimable_unrestaked_tickets.swap_remove(token_index);

                            let unrestaked_vst_amount = JitoRestakingVaultService::new(
                                vault_program.to_account_info(),
                                vault_account.to_account_info(),
                                vault_config.to_account_info(),
                                vault_vrt_mint.to_account_info(),
                                token_program.to_account_info(),
                                vault_vst_mint.to_account_info(),
                                token_program.to_account_info(),
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
                            .withdraw(
                                vault_withdrawal_ticket,
                                vault_withdrawal_ticket_token_account,
                                fund_supported_token_account,
                                vault_fee_receipt_token_account,
                                vault_program_fee_wallet_vrt_account,
                                &ctx.fund_account.as_ref(),
                                system_program,
                            )?;

                            match unused_claimable_unrestaked_tickets.first() {
                                Some(next_ticket) => {
                                    return Ok(Some(command.with_required_accounts([
                                        (vault_program.key(), false),
                                        (vault_account.key(), false),
                                        (vault_config.key(), false),
                                        (vault_vrt_mint.key(), false),
                                        (vault_vst_mint.key(), false),
                                        (fund_supported_token_account.key(), false),
                                        (fund_receipt_token_account.key(), false),
                                        (vault_fee_receipt_token_account.key(), false),
                                        (vault_program_fee_wallet_vrt_account.key(), false),
                                        (vault_update_state_tracker.key(), false),
                                        (
                                            vault_update_state_tracker_prepare_for_delaying.key(),
                                            false,
                                        ),
                                        (token_program.key(), false),
                                        (system_program.key(), false),
                                        (next_ticket.withdrawal_ticket_account, false),
                                        (next_ticket.withdrawal_ticket_token_account, false),
                                    ])))
                                }
                                None => {
                                    let normalized_token =
                                        &ctx.fund_account.normalized_token.as_ref().unwrap();
                                    if &restaking_vault.supported_token_mint
                                        == &normalized_token.mint
                                    {
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
                                        command.state =
                                            ClaimUnrestakedVSTCommandState::SetupDenormalize(
                                                unrestaked_vst_amount,
                                            );
                                        return Ok(Some(command.with_required_accounts([
                                            (normalized_token_pool_address, false),
                                            (normalized_token.mint, false),
                                            (normalized_token_account, false),
                                        ])));
                                    }
                                }
                            }
                        }
                        _ => err!(errors::ErrorCode::OperationCommandExecutionFailedException)?,
                    }
                }
                ClaimUnrestakedVSTCommandState::SetupDenormalize(total_denormalize_amount) => {
                    let [normalized_token_pool_address, normalized_token_mint, normalized_token_account, remaining_accounts @ ..] = accounts  else {
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
                    // let supported_tokens = ctx
                    //     .fund_account
                    //     .supported_tokens
                    //     .iter()
                    //     .filter_map(|t| {
                    //         if normalized_token_pool_account.has_supported_token(&t.mint)
                    //             && t.mint != normalized_token_mint.key()
                    //         {
                    //             let reserved_amount_as_sol = pricing_service
                    //                 .get_token_amount_as_sol(&t.mint, t.operation_reserved_amount)
                    //                 .unwrap();
                    //             Some((t, reserved_amount_as_sol))
                    //         } else {
                    //             None
                    //         }
                    //     })
                    //     .collect::<Vec<_>>();


                    // for (supported_token, reserved_token_amount_as_sol) in &supported_tokens {
                    //
                    // }

                }


                _ => (),
            }
        }
        if self.items.len() > 1 {
            return Ok(Some(
                ClaimUnrestakedVSTCommand::new_init(self.items[1..].to_vec())
                    .with_required_accounts([]),
            ));
        }
        Ok(None)
    }
}
