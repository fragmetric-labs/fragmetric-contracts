use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::errors::VaultError;
use crate::states::{SRTExchangeRate, VaultAccount};

#[event_cpi]
#[derive(Accounts)]
pub struct SolvManagerContext<'info> {
    pub solv_manager: Signer<'info>,
    /// CHECK: ..
    pub solv_protocol_wallet: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [VaultAccount::SEED, vault_receipt_token_mint.key().as_ref()],
        bump = vault_account.load()?.get_bump(),
        has_one = solv_manager @ VaultError::VaultAdminMismatchError,
        has_one = solv_protocol_wallet @ VaultError::SolvProtocolWalletMismatchError,
        has_one = vault_supported_token_mint @ VaultError::VaultSupportedTokenMintMismatchError,
        has_one = solv_receipt_token_mint @ VaultError::SolvReceiptTokenMintMismatchError,
        constraint = vault_account.load()?.is_latest_version() @ VaultError::InvalidAccountDataVersionError,
    )]
    pub vault_account: AccountLoader<'info, VaultAccount>,

    #[account(mut)]
    pub vault_receipt_token_mint: Account<'info, Mint>,
    pub vault_supported_token_mint: Account<'info, Mint>,
    pub solv_receipt_token_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = vault_receipt_token_mint,
        associated_token::authority = vault_account,
    )]
    pub vault_vault_receipt_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = vault_supported_token_mint,
        associated_token::authority = vault_account,
    )]
    pub vault_vault_supported_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = solv_receipt_token_mint,
        associated_token::authority = vault_account,
    )]
    pub vault_solv_receipt_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = vault_supported_token_mint,
        associated_token::authority = solv_protocol_wallet,
    )]
    pub solv_protocol_wallet_vault_supported_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = solv_receipt_token_mint,
        associated_token::authority = solv_protocol_wallet,
    )]
    pub solv_protocol_wallet_solv_receipt_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn process_deposit(ctx: Context<SolvManagerContext>) -> Result<()> {
    let SolvManagerContext {
        vault_account,
        vault_supported_token_mint,
        vault_vault_supported_token_account,
        solv_protocol_wallet_vault_supported_token_account,
        token_program,
        ..
    } = ctx.accounts;

    let vault = vault_account.load()?;

    let vst_amount = vault.get_vst_operation_reserved_amount();
    if vst_amount == 0 {
        // nothing to deposit
        // => just skip deposit
        return Ok(());
    }

    // TODO/phase3: CPI call to the Solv protocol - now just transfer
    anchor_spl::token::transfer_checked(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            anchor_spl::token::TransferChecked {
                from: vault_vault_supported_token_account.to_account_info(),
                mint: vault_supported_token_mint.to_account_info(),
                to: solv_protocol_wallet_vault_supported_token_account.to_account_info(),
                authority: vault_account.to_account_info(),
            },
            &[&vault.get_seeds()],
        ),
        vst_amount,
        vault_supported_token_mint.decimals,
    )?;

    drop(vault);

    let mut vault = vault_account.load_mut()?;

    // TODO/phase3: calculate ∆vault_solv_receipt_token_account.amount
    let srt_amount = vault.get_srt_operation_receivable_amount_for_deposit(vst_amount)?;
    vault.deposit_vst(vst_amount, srt_amount)?;

    // require_gte!(
    //     vault_solv_receipt_token_account.amount,
    //     vault.get_srt_total_reserved_amount(),
    // );

    Ok(())
}

// TODO/phase3: deprecate
pub fn process_confirm_deposit(
    ctx: Context<SolvManagerContext>,
    srt_amount: u64,
    srt_exchange_rate: SRTExchangeRate,
) -> Result<()> {
    let SolvManagerContext {
        vault_account,
        vault_solv_receipt_token_account,
        ..
    } = ctx.accounts;

    let mut vault = vault_account.load_mut()?;

    vault.resolve_srt_receivables(srt_amount, srt_exchange_rate)?;

    require_gte!(
        vault_solv_receipt_token_account.amount,
        vault.get_srt_total_reserved_amount(),
    );

    Ok(())
}

pub fn process_request_withdrawal(ctx: Context<SolvManagerContext>) -> Result<()> {
    let SolvManagerContext {
        vault_account,
        vault_receipt_token_mint,
        solv_receipt_token_mint,
        vault_vault_receipt_token_account,
        vault_solv_receipt_token_account,
        solv_protocol_wallet_solv_receipt_token_account,
        token_program,
        ..
    } = ctx.accounts;

    let (vrt_amount_to_burn, srt_amount_to_withdraw) =
        vault_account.load_mut()?.start_withdrawal_requests()?;

    if vrt_amount_to_burn > 0 {
        anchor_spl::token::burn(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                anchor_spl::token::Burn {
                    mint: vault_receipt_token_mint.to_account_info(),
                    from: vault_vault_receipt_token_account.to_account_info(),
                    authority: vault_account.to_account_info(),
                },
                &[&vault_account.load()?.get_seeds()],
            ),
            vrt_amount_to_burn,
        )?;
    }

    // TODO/phase3: CPI call to the Solv protocol - now just transfer
    if srt_amount_to_withdraw > 0 {
        anchor_spl::token::transfer_checked(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                anchor_spl::token::TransferChecked {
                    from: vault_solv_receipt_token_account.to_account_info(),
                    mint: solv_receipt_token_mint.to_account_info(),
                    to: solv_protocol_wallet_solv_receipt_token_account.to_account_info(),
                    authority: vault_account.to_account_info(),
                },
                &[&vault_account.load()?.get_seeds()],
            ),
            srt_amount_to_withdraw,
            solv_receipt_token_mint.decimals,
        )?;
    }

    Ok(())
}

pub fn process_withdraw(
    ctx: Context<SolvManagerContext>,
    srt_amount: u64,
    vst_amount: u64,
    srt_exchange_rate: SRTExchangeRate,
) -> Result<()> {
    let SolvManagerContext {
        vault_account,
        vault_vault_supported_token_account,
        ..
    } = ctx.accounts;

    // TODO/phase3: CPI call to the solv protocol - now assumes that solv protocol already sent VST to vault's ATA

    let mut vault = vault_account.load_mut()?;

    vault.complete_withdrawal_requests(srt_amount, vst_amount, srt_exchange_rate)?;

    require_gte!(
        vault_vault_supported_token_account.amount,
        vault.get_vst_total_reserved_amount(),
    );

    Ok(())
}
