use anchor_lang::prelude::*;
use jito_vault_core::{vault::Vault, vault_operator_delegation::VaultOperatorDelegation, vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket};
use jito_bytemuck::AccountDeserialize;

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


pub struct JitoRestakingVaultWithdrawalContext<'info> {
    pub vault_program: AccountInfo<'info>,
    pub vault_config: AccountInfo<'info>,
    pub vault: AccountInfo<'info>,
    pub vault_receipt_token_mint: AccountInfo<'info>,
    pub vault_receipt_token_program: AccountInfo<'info>,
    pub vault_supported_token_mint: AccountInfo<'info>,
    pub vault_supported_token_program: AccountInfo<'info>,
    pub vault_supported_token_account: AccountInfo<'info>,
    // pub vault_program_base_account1: AccountInfo<'info>,
}


// impl JitoRestakingVaultWithdrawalContext<'info> {
//     // pub fn get_vault_withdrawal_tickets(vault_withdrawal_tickets: &[AccountInfo]) -> Result<[VaultStakerWithdrawalTicket]>{
//     //     let tickets= Vec::new();
//     //     for vault_withdrawal_ticket in vault_withdrawal_tickets {
//     //         let ticket_data_ref = vault_withdrawal_ticket.data.borrow();
//     //         let ticket_data = ticket_data_ref.as_ref();
//     //         let ticket = VaultStakerWithdrawalTicket::try_from_slice_unchecked(ticket_data)?;
//     //         tickets.push(ticket);
//     //     }
//     //     Ok(tickets)
//     // }
// }
