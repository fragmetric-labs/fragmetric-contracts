use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::{invoke_signed};
use anchor_spl::mint;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount};
use jito_vault_sdk;
use super::*;

// #[derive(Debug, Clone)]
// pub struct Jito;
//
// impl Id for Jito {
//     fn id() -> Pubkey {
//         JitoRestakingProtocol::JITO_VAULT_PROGRAM_ID
//     }
// }

pub struct JitoRestakingVaultContext<'info> {
    // #[account(address = JitoRestakingProtocol::JITO_VAULT_PROGRAM_ID)]
    pub vault_program: AccountInfo<'info>,
    
    // #[account(mut, address = JitoRestakingProtocol::JITO_VAULT_CONFIG_ADDRESS)]
    pub vault_config: AccountInfo<'info>,
    
    // #[account(mut, address = JitoRestakingProtocol::JITO_VAULT_ADDRESS)]
    pub vault: AccountInfo<'info>,

    // #[account(mut, address = JitoRestakingProtocol::JITO_VAULT_RECEIPT_TOKEN)]
    pub vault_receipt_token_mint: AccountInfo<'info>, // InterfaceAccount<'info, Mint>,

    pub vault_receipt_token_program: AccountInfo<'info>, // Program<'info, Token>,

    // #[account(mut, address = JitoRestakingProtocol::JITO_VAULT_SUPPORTED_TOKEN)]
    pub vault_supported_token_mint: AccountInfo<'info>, // InterfaceAccount<'info, Mint>,

    pub vault_supported_token_program: AccountInfo<'info>, // Program<'info, Token>,
    
    // #[account(
    //     mut,
    //     associated_token::mint = vault_supported_token,
    //     associated_token::token_program = token_program,
    //     associated_token::authority = vault,
    // )]
    pub vault_supported_token_account: AccountInfo<'info>, // Box<InterfaceAccount<'info, TokenAccount>>,

    // #[account(mut)]
    // pub vault_update_state_tracker: AccountInfo<'info>, // UncheckedAccount<'info>,
    
    // #[account(
    //     mut,
    //     associated_token::mint = vault_supported_token,
    //     associated_token::token_program = token_program,
    //     associated_token::authority = user,
    // )]
    // pub user_supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    // #[account(
    //     mut,
    //     associated_token::mint = vault_receipt_token,
    //     associated_token::token_program = token_program,
    //     associated_token::authority = user,
    // )]
    // pub user_vault_receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
}

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

pub fn deposit<'info>(
    ctx: &JitoRestakingVaultContext<'info>,

    supported_token_account: &AccountInfo<'info>, // &InterfaceAccount<'info, TokenAccount>,
    supported_token_amount_in: u64,

    vault_receipt_token_account: &AccountInfo<'info>, // &InterfaceAccount<'info, TokenAccount>,
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

        vault_receipt_token_account.key,

        Some(signer.key),

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
            ctx.vault_receipt_token_program.to_account_info(),
        ],
        signer_seeds,
    )?;

    Ok(())
}

