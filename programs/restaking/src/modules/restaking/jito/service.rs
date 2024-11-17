use crate::modules::restaking::jito::{
    JitoRestakingVault, JitoRestakingVaultContext, JitoVaultOperatorDelegation,
};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use jito_bytemuck::AccountDeserialize;
use jito_vault_core::vault_operator_delegation::VaultOperatorDelegation;
use jito_vault_core::{vault::Vault, vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket};

impl JitoRestakingVault {
    pub const VAULT_BASE_ACCOUNT1_SEED: &'static [u8] = b"vault_base_account1";
    pub const VAULT_BASE_ACCOUNT2_SEED: &'static [u8] = b"vault_base_account2";
    pub fn get_vault_operator_delegation_key(
        ctx: JitoRestakingVaultContext,
        operator: &Pubkey,
    ) -> Pubkey {
        let (vault_operator_delegation_key, _, _) = VaultOperatorDelegation::find_program_address(
            ctx.vault_program.key,
            ctx.vault.key,
            operator,
        );
        vault_operator_delegation_key
    }

    pub fn get_status(
        vault: AccountInfo,
        vault_operators_delegation: &[AccountInfo],
    ) -> Result<JitoRestakingVault> {
        let vault_data_ref = vault.data.borrow();
        let vault_data = vault_data_ref.as_ref();
        let vault = Vault::try_from_slice_unchecked(vault_data)?;

        // Vault's status
        let vault_receipt_token_withdrawal_pending_amount = vault.vrt_cooling_down_amount();
        let vault_receipt_token_enqueued_for_withdrawal_amount =
            vault.vrt_enqueued_for_cooldown_amount();
        let vault_receipt_token_claimable_amount = vault.vrt_ready_to_claim_amount();
        let vault_receipt_token_unrestake_pending_amount =
            vault_receipt_token_withdrawal_pending_amount
                .checked_add(vault_receipt_token_enqueued_for_withdrawal_amount)
                .unwrap();
        let vault_supported_token_restaked_amount = vault.tokens_deposited();

        // Vault::DelegationState's status
        let vault_delegation_state = vault.delegation_state;
        let supported_token_cooling_down_amount = vault_delegation_state.cooling_down_amount();
        let supported_token_enqueued_for_undelegate_amount =
            vault_delegation_state.enqueued_for_cooldown_amount();
        let supported_token_undelegate_pending_amount = supported_token_cooling_down_amount
            .checked_add(supported_token_enqueued_for_undelegate_amount)
            .unwrap();
        let supported_token_delegated_amount = vault_delegation_state.staked_amount();

        // Check Validation
        let operator_count = vault.operator_count();
        if vault_operators_delegation.len() != operator_count as usize {
            msg!("the number of vault operators delegation does not match the operator count in the vault.");
            return Err(ProgramError::InvalidAccountData.into());
        }

        // VaultOperatorDelegation's status
        let mut vault_operators_delegation_status = Vec::new();
        for vault_operator_delegation in vault_operators_delegation {
            if vault_operator_delegation.owner != &vault.operator_admin {
                msg!("owner and operator admin do not match.");
                return Err(ProgramError::InvalidAccountData.into());
            }
            let vault_operator_delegation_data_ref = vault_operator_delegation.data.borrow();
            let vault_operator_delegation_data = vault_operator_delegation_data_ref.as_ref();
            let vault_operator_delegation =
                VaultOperatorDelegation::try_from_slice_unchecked(vault_operator_delegation_data)?;

            let vault_operator_delegation_state = vault_operator_delegation.delegation_state;
            let operator_supported_token_delegated_amount =
                vault_operator_delegation_state.staked_amount();
            let operator_supported_token_cooling_down_amount =
                vault_operator_delegation_state.cooling_down_amount();
            let operator_supported_token_enqueued_for_undelegate_amount =
                vault_operator_delegation_state.enqueued_for_cooldown_amount();
            let operator_supported_token_undelegate_pending_amount =
                operator_supported_token_cooling_down_amount
                    .checked_add(operator_supported_token_enqueued_for_undelegate_amount)
                    .unwrap();

            vault_operators_delegation_status.push(JitoVaultOperatorDelegation {
                operator_supported_token_delegated_amount,
                operator_supported_token_undelegate_pending_amount,
            });
        }

        Ok(Self {
            vault_receipt_token_claimable_amount,
            vault_receipt_token_unrestake_pending_amount,
            vault_supported_token_restaked_amount,
            supported_token_undelegate_pending_amount,
            supported_token_delegated_amount,
            vault_operators_delegation_status,
        })
    }
}

pub fn deposit<'info>(
    ctx: &JitoRestakingVaultContext<'info>,
    supported_token_account: &AccountInfo<'info>,
    supported_token_amount_in: u64,
    vault_fee_receipt_token_account: &AccountInfo<'info>,
    vault_receipt_token_account: &AccountInfo<'info>,
    vault_receipt_token_min_amount_out: u64,
    signer: &AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    let mint_to_ix = jito_vault_sdk::sdk::mint_to(
        ctx.vault_program.key,
        ctx.vault_config.key,
        ctx.vault.key,
        ctx.vault_receipt_token_mint.key,
        signer.key,
        supported_token_account.key,
        ctx.vault_supported_token_account.key,
        vault_receipt_token_account.key,
        // TODO: read fee admin ata from vault state account
        vault_fee_receipt_token_account.key,
        None,
        supported_token_amount_in,
        vault_receipt_token_min_amount_out,
    );

    invoke_signed(
        &mint_to_ix,
        &[
            ctx.vault_program.clone(),
            ctx.vault_config.clone(),
            ctx.vault.clone(),
            ctx.vault_receipt_token_mint.clone(),
            signer.clone(),
            supported_token_account.clone(),
            ctx.vault_supported_token_account.clone(),
            vault_receipt_token_account.clone(),
            vault_fee_receipt_token_account.clone(),
            ctx.vault_receipt_token_program.clone(),
        ],
        signer_seeds,
    )?;

    Ok(())
}

pub fn delegation<'info>(
    ctx: &JitoRestakingVaultContext<'info>,
    vault_operator: &AccountInfo<'info>,
    vault_operator_delegation: &AccountInfo<'info>,
    signer: &AccountInfo<'info>, // jito_vault_delegation_admin
    signer_seeds: &[&[&[u8]]],
    supported_token_delegation_amount_in: u64,
) -> Result<()> {
    let add_delegation_ix = jito_vault_sdk::sdk::add_delegation(
        ctx.vault_program.key,
        ctx.vault_config.key,
        ctx.vault.key,
        vault_operator.key,
        vault_operator_delegation.key,
        signer.key,
        supported_token_delegation_amount_in,
    );

    invoke_signed(
        &add_delegation_ix,
        &[
            ctx.vault_program.clone(),
            ctx.vault_config.clone(),
            ctx.vault.clone(),
            vault_operator.clone(),
            vault_operator_delegation.clone(),
            signer.clone(),
        ],
        signer_seeds,
    )?;

    Ok(())
}

pub fn undelegation<'info>(
    ctx: &JitoRestakingVaultContext<'info>,
    vault_operator: &AccountInfo<'info>,
    vault_operator_delegation: &AccountInfo<'info>,
    signer: &AccountInfo<'info>, // jito_vault_delegation_admin
    signer_seeds: &[&[&[u8]]],
    supported_token_delegation_amount_out: u64,
) -> Result<()> {
    let cooldown_delegation_ix = jito_vault_sdk::sdk::cooldown_delegation(
        ctx.vault_program.key,
        ctx.vault_config.key,
        ctx.vault.key,
        vault_operator.key,
        vault_operator_delegation.key,
        signer.key,
        supported_token_delegation_amount_out,
    );

    invoke_signed(
        &cooldown_delegation_ix,
        &[
            ctx.vault_program.clone(),
            ctx.vault_config.clone(),
            ctx.vault.clone(),
            vault_operator.clone(),
            vault_operator_delegation.clone(),
            signer.clone(),
        ],
        signer_seeds,
    )?;

    Ok(())
}

pub fn request_withdraw<'info>(
    ctx: &JitoRestakingVaultContext<'info>,
    operator: &Signer<'info>,
    vault_withdrawal_ticket: &AccountInfo<'info>,
    vault_withdrawal_ticket_token_account: &AccountInfo<'info>,
    vault_receipt_token_account: &AccountInfo<'info>,
    vault_base_account: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    associated_token_program: &AccountInfo<'info>,
    signer: &AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    vrt_token_amount_out: u64,
) -> Result<()> {
    // TODO create token account
    anchor_spl::associated_token::create(CpiContext::new(
        associated_token_program.clone(),
        anchor_spl::associated_token::Create {
            payer: operator.to_account_info(),
            associated_token: vault_withdrawal_ticket_token_account.clone(),
            authority: vault_withdrawal_ticket.clone(),
            mint: ctx.vault_receipt_token_mint.clone(),
            system_program: system_program.clone(),
            token_program: ctx.vault_receipt_token_program.clone(),
        },
    ))?;

    let rent = Rent::get()?;
    let current_lamports = vault_withdrawal_ticket.lamports();
    let space = 8 + std::mem::size_of::<VaultStakerWithdrawalTicket>();
    let required_lamports = rent
        .minimum_balance(space as usize)
        .max(1)
        .saturating_sub(current_lamports);

    if required_lamports > 0 {
        anchor_lang::system_program::transfer(
            CpiContext::new(
                system_program.clone(),
                anchor_lang::system_program::Transfer {
                    from: operator.to_account_info(),
                    to: vault_withdrawal_ticket.clone(),
                },
            ),
            required_lamports,
        )?;
    }

    let enqueue_withdraw_ix = jito_vault_sdk::sdk::enqueue_withdrawal(
        ctx.vault_program.key,
        ctx.vault_config.key,
        ctx.vault.key,
        vault_withdrawal_ticket.key,
        vault_withdrawal_ticket_token_account.key,
        signer.key,
        vault_receipt_token_account.key,
        vault_base_account.key,
        vrt_token_amount_out,
    );

    invoke_signed(
        &enqueue_withdraw_ix,
        &[
            ctx.vault_program.clone(),
            ctx.vault_config.clone(),
            ctx.vault.clone(),
            vault_withdrawal_ticket.clone(),
            vault_withdrawal_ticket_token_account.clone(),
            signer.clone(),
            vault_receipt_token_account.clone(),
            vault_base_account.clone(),
            ctx.vault_receipt_token_program.clone(),
            system_program.clone(),
        ],
        signer_seeds,
    )?;

    Ok(())
}

pub fn withdraw<'info>(
    ctx: &JitoRestakingVaultContext<'info>,
    vault_withdrawal_ticket: &AccountInfo<'info>,
    vault_withdrawal_ticket_token_account: &AccountInfo<'info>,
    fund_vault_supported_token_account: &AccountInfo<'info>,
    vault_fee_receipt_token_account: &AccountInfo<'info>,
    vault_program_fee_wallet_vrt_account: &AccountInfo<'info>,
    signer: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
) -> Result<()> {
    let enqueue_withdraw_ix = jito_vault_sdk::sdk::burn_withdrawal_ticket(
        ctx.vault_program.key,
        ctx.vault_config.key,
        ctx.vault.key,
        ctx.vault_supported_token_account.key,
        ctx.vault_receipt_token_mint.key,
        signer.key,
        fund_vault_supported_token_account.key,
        vault_withdrawal_ticket.key,
        vault_withdrawal_ticket_token_account.key,
        vault_fee_receipt_token_account.key,
        vault_program_fee_wallet_vrt_account.key,
    );

    invoke_signed(
        &enqueue_withdraw_ix,
        &[
            ctx.vault_program.clone(),
            ctx.vault_config.clone(),
            ctx.vault.clone(),
            ctx.vault_supported_token_account.clone(),
            ctx.vault_receipt_token_mint.clone(),
            signer.clone(),
            fund_vault_supported_token_account.clone(),
            vault_withdrawal_ticket.clone(),
            vault_withdrawal_ticket_token_account.clone(),
            vault_fee_receipt_token_account.clone(),
            vault_program_fee_wallet_vrt_account.clone(),
            ctx.vault_receipt_token_program.clone(),
            system_program.clone(),
        ],
        &[],
    )?;

    Ok(())
}
