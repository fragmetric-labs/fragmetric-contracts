use crate::errors::ErrorCode;
use anchor_lang::prelude::*;
use jito_bytemuck::AccountDeserialize;
use jito_vault_core::{
    config::Config, vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket,
};

pub struct JitoVaultOperatorDelegation {
    pub operator_supported_token_delegated_amount: u64,
    pub operator_supported_token_undelegate_pending_amount: u64,
}

pub struct JitoRestakingVault {
    pub vault_receipt_token_claimable_amount: u64,
    pub vault_receipt_token_unrestake_pending_amount: u64,
    pub vault_supported_token_restaked_amount: u64,
    pub supported_token_undelegate_pending_amount: u64,
    pub supported_token_delegated_amount: u64,
    pub vault_operators_delegation_status: Vec<JitoVaultOperatorDelegation>,
}

pub struct JitoRestakingVaultContext<'info> {
    pub vault_program: AccountInfo<'info>,
    pub vault_config: AccountInfo<'info>,
    pub vault: AccountInfo<'info>,
    pub vault_receipt_token_mint: AccountInfo<'info>,
    pub vault_receipt_token_program: AccountInfo<'info>,
    pub vault_supported_token_mint: AccountInfo<'info>,
    pub vault_supported_token_program: AccountInfo<'info>,
    pub vault_supported_token_account: AccountInfo<'info>,
}

impl<'info> JitoRestakingVaultContext<'info> {
    pub fn get_ready_to_burn_withdrawal_tickets(
        &self,
        vault_withdrawal_tickets: &'info [AccountInfo<'info>],
        slot: u64,
    ) -> Result<Vec<&'info AccountInfo<'info>>> {
        let vault_config_data = &**self.vault_config.try_borrow_data()?;
        let vault_config = Config::try_from_slice_unchecked(vault_config_data)?;
        let epoch_length = vault_config.epoch_length();
        let mut tickets = Vec::new();
        for vault_withdrawal_ticket in vault_withdrawal_tickets {
            if vault_withdrawal_ticket.data_is_empty() && vault_withdrawal_ticket.lamports() == 0 {
                continue;
            }

            let ticket_data_ref = vault_withdrawal_ticket.data.borrow();
            let ticket_data = ticket_data_ref.as_ref();
            let ticket = VaultStakerWithdrawalTicket::try_from_slice_unchecked(ticket_data)?;
            if ticket.is_withdrawable(slot, epoch_length)? {
                tickets.push(vault_withdrawal_ticket);
            }
        }
        Ok(tickets)
    }

    pub fn check_ready_to_burn_withdrawal_ticket(
        &self,
        vault_withdrawal_ticket: &'info AccountInfo<'info>,
        slot: u64,
    ) -> Result<bool> {
        let vault_config_data = &**self.vault_config.try_borrow_data()?;
        let vault_config = Config::try_from_slice_unchecked(vault_config_data)?;
        let epoch_length = vault_config.epoch_length();
        if vault_withdrawal_ticket.data_is_empty() && vault_withdrawal_ticket.lamports() == 0 {
            return Err(Error::from(
                ErrorCode::RestakingVaultWithdrawalTicketNotWithdrawableError,
            ));
        }
        let ticket_data_ref = vault_withdrawal_ticket.data.borrow();
        let ticket_data = ticket_data_ref.as_ref();
        let ticket = VaultStakerWithdrawalTicket::try_from_slice_unchecked(ticket_data)?;
        if ticket.is_withdrawable(slot, epoch_length)? {
            Ok(true)
        } else {
            Err(Error::from(
                ErrorCode::RestakingVaultWithdrawalTicketNotWithdrawableError,
            ))
        }
    }

    pub fn check_withdrawal_ticket_is_empty(
        &self,
        vault_withdrawal_ticket: &'info AccountInfo<'info>,
    ) -> Result<bool> {
        if vault_withdrawal_ticket.data_is_empty() && vault_withdrawal_ticket.lamports() == 0 {
            Ok(true)
        } else {
            Err(Error::from(
                ErrorCode::RestakingVaultWithdrawalTicketAlreadyInitializedError,
            ))
        }
    }
}
