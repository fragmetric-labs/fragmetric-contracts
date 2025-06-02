use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::errors::VaultError;
use crate::states::VaultAccount;

#[event_cpi]
#[derive(Accounts)]
pub struct FundManagerContext<'info> {
    pub payer: Signer<'info>,
    pub fund_manager: Signer<'info>,

    #[account(
        mut,
        seeds = [VaultAccount::SEED, vault_receipt_token_mint.key().as_ref()],
        bump = vault_account.load()?.get_bump(),
        has_one = fund_manager @ VaultError::VaultAdminMismatchError,
        has_one = vault_supported_token_mint @ VaultError::VaultSupportedTokenMintMismatchError,
        constraint = vault_account.load()?.is_latest_version() @ VaultError::InvalidAccountDataVersionError,
    )]
    pub vault_account: AccountLoader<'info, VaultAccount>,

    #[account(mut)]
    pub vault_receipt_token_mint: Account<'info, Mint>,
    pub vault_supported_token_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = vault_receipt_token_mint,
        associated_token::authority = payer,
    )]
    pub payer_vault_receipt_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = vault_supported_token_mint,
        associated_token::authority = payer,
    )]
    pub payer_vault_supported_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = vault_supported_token_mint,
        associated_token::authority = vault_account,
    )]
    pub vault_vault_supported_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn process_deposit(ctx: Context<FundManagerContext>, vst_amount: u64) -> Result<()> {
    let FundManagerContext {
        payer,
        vault_account,
        vault_receipt_token_mint,
        vault_supported_token_mint,
        payer_vault_receipt_token_account,
        payer_vault_supported_token_account,
        vault_vault_supported_token_account,
        token_program,
        ..
    } = ctx.accounts;

    require_gt!(vst_amount, 0);

    let vrt_amount = vault_account.load_mut()?.mint_vrt(vst_amount)?;

    anchor_spl::token::transfer_checked(
        CpiContext::new(
            token_program.to_account_info(),
            anchor_spl::token::TransferChecked {
                from: payer_vault_supported_token_account.to_account_info(),
                mint: vault_supported_token_mint.to_account_info(),
                to: vault_vault_supported_token_account.to_account_info(),
                authority: payer.to_account_info(),
            },
        ),
        vst_amount,
        vault_supported_token_mint.decimals,
    )?;

    if vrt_amount > 0 {
        anchor_spl::token::mint_to(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                anchor_spl::token::MintTo {
                    mint: vault_receipt_token_mint.to_account_info(),
                    to: payer_vault_receipt_token_account.to_account_info(),
                    authority: vault_account.to_account_info(),
                },
                &[&vault_account.load()?.get_seeds()],
            ),
            vrt_amount,
        )?;
    }

    Ok(())
}

pub fn process_request_withdrawal(ctx: Context<FundManagerContext>, vrt_amount: u64) -> Result<()> {
    let FundManagerContext {
        payer,
        vault_account,
        vault_receipt_token_mint,
        payer_vault_receipt_token_account,
        token_program,
        ..
    } = ctx.accounts;

    require_gt!(vrt_amount, 0);

    let vrt_amount = vault_account
        .load_mut()?
        .enqueue_withdrawal_request(vrt_amount)?;

    anchor_spl::token::burn(
        CpiContext::new(
            token_program.to_account_info(),
            anchor_spl::token::Burn {
                mint: vault_receipt_token_mint.to_account_info(),
                from: payer_vault_receipt_token_account.to_account_info(),
                authority: payer.to_account_info(),
            },
        ),
        vrt_amount,
    )?;

    Ok(())
}

pub fn process_withdraw(ctx: Context<FundManagerContext>) -> Result<()> {
    let FundManagerContext {
        vault_account,
        vault_supported_token_mint,
        payer_vault_supported_token_account,
        vault_vault_supported_token_account,
        token_program,
        ..
    } = ctx.accounts;

    let vst_amount = vault_account.load_mut()?.claim_vst()?;

    if vst_amount > 0 {
        anchor_spl::token::transfer_checked(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                anchor_spl::token::TransferChecked {
                    from: vault_vault_supported_token_account.to_account_info(),
                    mint: vault_supported_token_mint.to_account_info(),
                    to: payer_vault_supported_token_account.to_account_info(),
                    authority: vault_account.to_account_info(),
                },
                &[&vault_account.load()?.get_seeds()],
            ),
            vst_amount,
            vault_supported_token_mint.decimals,
        )?;
    }

    Ok(())
}
