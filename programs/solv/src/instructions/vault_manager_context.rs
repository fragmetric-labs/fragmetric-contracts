use anchor_lang::prelude::*;
use anchor_spl::token::spl_token;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::errors::VaultError;
use crate::states::VaultAccount;

#[event_cpi]
#[derive(Accounts)]
pub struct VaultManagerVaultAccountInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub vault_manager: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = VaultAccount::get_size(),
        seeds = [VaultAccount::SEED, vault_receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub vault_account: AccountLoader<'info, VaultAccount>,

    #[account(
        mut,
        constraint = vault_receipt_token_mint.supply == 0,
    )]
    pub vault_receipt_token_mint: Account<'info, Mint>,
    pub vault_supported_token_mint: Account<'info, Mint>,
    pub solv_receipt_token_mint: Account<'info, Mint>,

    #[account(
        associated_token::mint = vault_receipt_token_mint,
        associated_token::authority = vault_account,
    )]
    pub vault_receipt_token_account: Account<'info, TokenAccount>,

    #[account(
        associated_token::mint = vault_supported_token_mint,
        associated_token::authority = vault_account,
    )]
    pub vault_supported_token_account: Account<'info, TokenAccount>,

    #[account(
        associated_token::mint = solv_receipt_token_mint,
        associated_token::authority = vault_account,
    )]
    pub solv_receipt_token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct VaultManagerVaultAccountUpdateContext<'info> {
    pub vault_manager: Signer<'info>,

    #[account(
        mut,
        seeds = [VaultAccount::SEED, vault_receipt_token_mint.key().as_ref()],
        bump = vault_account.load()?.get_bump(),
        has_one = vault_manager @ VaultError::VaultAdminMismatchError,
        constraint = vault_account.load()?.is_initialized() @ VaultError::InvalidAccountDataVersionError,
    )]
    pub vault_account: AccountLoader<'info, VaultAccount>,

    pub vault_receipt_token_mint: Account<'info, Mint>,
    pub vault_supported_token_mint: Account<'info, Mint>,
    pub solv_receipt_token_mint: Account<'info, Mint>,
}

pub fn process_initialize_vault_account(
    ctx: Context<VaultManagerVaultAccountInitialContext>,
) -> Result<()> {
    let VaultManagerVaultAccountInitialContext {
        vault_manager,
        vault_account,
        vault_receipt_token_mint,
        vault_supported_token_mint,
        solv_receipt_token_mint,
        token_program,
        ..
    } = ctx.accounts;

    vault_account.load_init()?.initialize(
        vault_manager,
        vault_receipt_token_mint,
        vault_supported_token_mint,
        solv_receipt_token_mint,
        ctx.bumps.vault_account,
    )?;

    let current_authority = vault_receipt_token_mint.mint_authority.unwrap_or_default();
    if current_authority != vault_account.key() {
        // Expects vault manager to be current mint authority
        require_keys_eq!(current_authority, vault_manager.key());

        anchor_spl::token::set_authority(
            CpiContext::new(
                token_program.to_account_info(),
                anchor_spl::token::SetAuthority {
                    current_authority: vault_manager.to_account_info(),
                    account_or_mint: vault_receipt_token_mint.to_account_info(),
                },
            ),
            spl_token::instruction::AuthorityType::MintTokens,
            Some(vault_account.key()),
        )?;
    }

    Ok(())
}

pub fn process_update_vault_account_if_needed(
    ctx: Context<VaultManagerVaultAccountUpdateContext>,
) -> Result<()> {
    let VaultManagerVaultAccountUpdateContext {
        vault_manager,
        vault_account,
        vault_receipt_token_mint,
        vault_supported_token_mint,
        solv_receipt_token_mint,
        ..
    } = ctx.accounts;

    vault_account.load_mut()?.update_if_needed(
        vault_manager,
        vault_receipt_token_mint,
        vault_supported_token_mint,
        solv_receipt_token_mint,
    )?;

    Ok(())
}
