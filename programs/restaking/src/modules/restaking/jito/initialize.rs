use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::{invoke_signed};
use anchor_spl::mint;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount};
use jito_vault_sdk;
use super::*;
//
//
// fn initialize_vault_update_state_tracker(ctx: &JitoRestakingVaultContext) -> Result<()> {
//     let initialize_vault_update_state_tracker_ix = jito_vault_sdk::sdk::initialize_vault_update_state_tracker(
//         ctx.program.key,
//         ctx.config.key,
//         ctx.vault.key,
//         ctx.vault_update_state_tracker.key,
//         ctx.user.key,
//         TryFrom::try_from(0u8).unwrap(),
//     );
//
//     invoke(
//         &initialize_vault_update_state_tracker_ix,
//         &[ctx.program.to_account_info(),
//             ctx.config.to_account_info(),
//             ctx.vault.to_account_info(),
//             ctx.vault_update_state_tracker.to_account_info(),
//             ctx.user.to_account_info(),
//             ctx.system_program.to_account_info()
//         ],
//     )?;
//
//     Ok(())
// }
//
// fn close_vault_update_state_tracker(ctx: &JitoRestakingVaultContext) -> Result<()> {
//     let close_vault_update_state_tracker_ix = jito_vault_sdk::sdk::close_vault_update_state_tracker(
//         ctx.program.key,
//         ctx.config.key,
//         ctx.vault.key,
//         ctx.vault_update_state_tracker.key,
//         ctx.user.key,
//         Clock::get()?.slot.checked_div(432000).unwrap(),
//     );
//
//     invoke(
//         &close_vault_update_state_tracker_ix,
//         &[ctx.program.to_account_info(),
//             ctx.config.to_account_info(),
//             ctx.vault.to_account_info(),
//             ctx.vault_update_state_tracker.to_account_info(),
//             ctx.user.to_account_info(),
//             ctx.system_program.to_account_info()
//         ],
//     )?;
//
//     Ok(())
// }
//
// fn update_vault_balance(ctx: &JitoRestakingVaultContext) -> Result<()> {
//     let update_vault_balance_ix = jito_vault_sdk::sdk::update_vault_balance(
//         ctx.program.key,
//         ctx.config.key,
//         ctx.vault.key,
//         &ctx.vault_supported_token_account.key(),
//         ctx.vault_receipt_token.key,
//         &ctx.user_receipt_token_account.key(),
//         ctx.token_program.key,
//     );
//
//     invoke(
//         &update_vault_balance_ix,
//         &[
//             ctx.program.to_account_info(),
//             ctx.config.to_account_info(),
//             ctx.vault.to_account_info(),
//             ctx.vault_supported_token_account.to_account_info(),
//             ctx.vault_receipt_token.to_account_info(),
//             ctx.user_receipt_token_account.to_account_info(),
//             ctx.token_program.to_account_info(),
//         ],
//     )?;
//
//     Ok(())
// }


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
