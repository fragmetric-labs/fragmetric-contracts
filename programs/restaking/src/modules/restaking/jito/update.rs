use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::{invoke_signed};
use anchor_spl::mint;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount};
use jito_vault_sdk;
use super::*;


fn initialize_vault_update_state_tracker<'info>(
    ctx: &JitoRestakingVaultContext<'info>,
    signer: &AccountInfo<'info>,
    vault_update_state_tracker: AccountInfo<'info>,
    system_program: &AccountInfo<'info>
) -> Result<()> {
    let initialize_vault_update_state_tracker_ix = jito_vault_sdk::sdk::initialize_vault_update_state_tracker(
        ctx.vault_program.key,
        ctx.vault_config.key,
        ctx.vault.key,
        &vault_update_state_tracker.key,
        signer.key,
        TryFrom::try_from(0u8).unwrap(), // WithdrawalAllocationMethod
    );

    invoke_signed(
        &initialize_vault_update_state_tracker_ix,
        &[
            ctx.vault_program.clone(),
            ctx.vault_config.clone(),
            ctx.vault.clone(),
            vault_update_state_tracker.clone(),
            signer.clone(),
            system_program.clone()
        ],
        &[],
    )?;

    Ok(())
}

fn close_vault_update_state_tracker<'info>(
    ctx: &JitoRestakingVaultContext<'info>,
    signer: &AccountInfo<'info>,
    vault_update_state_tracker: AccountInfo<'info>,
    ncn_epoch: u64
) -> Result<()> {
    let close_vault_update_state_tracker_ix = jito_vault_sdk::sdk::close_vault_update_state_tracker(
        ctx.vault_program.key,
        ctx.vault_config.key,
        ctx.vault.key,
        &vault_update_state_tracker.key,
        signer.key,
        ncn_epoch, // Clock::get()?.slot.checked_div(432000).unwrap(), need to change 432000 -> config.epoch_length
    );

    invoke_signed(
        &close_vault_update_state_tracker_ix,
        &[
            ctx.vault_program.clone(),
            ctx.vault_config.clone(),
            ctx.vault.clone(),
            vault_update_state_tracker.clone(),
            signer.clone(),
        ],
        &[],
    )?;

    Ok(())
}

fn update_vault_balance<'info>(
    ctx: &JitoRestakingVaultContext<'info>,
    vault_fee_receipt_token_account: &AccountInfo<'info>
) -> Result<()> {
    let update_vault_balance_ix = jito_vault_sdk::sdk::update_vault_balance(
        ctx.vault_program.key,
        ctx.vault_config.key,
        ctx.vault.key,
        &ctx.vault_supported_token_account.key(),
        ctx.vault_receipt_token_mint.key,
        &vault_fee_receipt_token_account.key(),
        ctx.vault_receipt_token_program.key,
    );

    invoke_signed(
        &update_vault_balance_ix,
        &[
            ctx.vault_program.clone(),
            ctx.vault_config.clone(),
            ctx.vault.clone(),
            ctx.vault_supported_token_account.clone(),
            ctx.vault_receipt_token_mint.clone(),
            vault_fee_receipt_token_account.clone(),
            ctx.vault_receipt_token_program.clone(),
        ],
        &[],
    )?;

    Ok(())
}


pub fn initialize_vault_operator_delegation<'info>(
    ctx: JitoRestakingVaultContext<'info>,
    vault_operator: &AccountInfo<'info>,
    vault_operator_vault_ticket: &AccountInfo<'info>,
    vault_operator_delegation: &AccountInfo<'info>,
    signer: &AccountInfo<'info>, // jito_vault_delegation_admin
    signer_seeds: &[&[&[u8]]],
    system_program: &AccountInfo<'info>,
) -> anchor_lang::Result<()> {
    let add_delegation_ix = jito_vault_sdk::sdk::initialize_vault_operator_delegation(
        ctx.vault_program.key,
        ctx.vault_config.key,
        ctx.vault.key,
        vault_operator.key,
        vault_operator_vault_ticket.key,
        vault_operator_delegation.key,
        signer.key, // vault_operator_admin
        signer.key, // payer
    );

    invoke_signed(
        &add_delegation_ix,
        &[
            ctx.vault_program.clone(),
            ctx.vault_config.clone(),
            ctx.vault.clone(),
            vault_operator.clone(),
            vault_operator_vault_ticket.clone(),
            vault_operator_delegation.clone(),
            signer.clone(), // vault_operator_admin
            signer.clone(), // payer
            system_program.clone()
        ],
        signer_seeds,
    )?;

    Ok(())
}
