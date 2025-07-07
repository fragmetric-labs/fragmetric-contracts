use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::errors::VaultError;
use crate::events;
use crate::states::VaultAccount;

#[event_cpi]
#[derive(Accounts)]
pub struct SolvManagerContext<'info> {
    pub solv_manager: Signer<'info>,
    /// CHECK: ..
    #[account(constraint = solv_protocol_wallet.key() != System::id())]
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

    pub vault_receipt_token_mint: Account<'info, Mint>,
    pub vault_supported_token_mint: Account<'info, Mint>,
    pub solv_receipt_token_mint: Account<'info, Mint>,

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

pub fn process_confirm_deposits(
    ctx: &mut Context<SolvManagerContext>,
) -> Result<Option<events::SolvManagerConfirmedDeposits>> {
    let SolvManagerContext {
        solv_manager,
        solv_protocol_wallet,
        vault_account,
        vault_receipt_token_mint,
        vault_supported_token_mint,
        solv_receipt_token_mint,
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
        return Ok(None);
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

    let (one_srt_as_micro_vst, deducted_vst_deposit_fee_amount, expected_srt_amount) =
        vault_account.load_mut()?.deposit_vst(vst_amount)?;

    Ok(Some(events::SolvManagerConfirmedDeposits {
        vault: vault_account.key(),
        solv_protocol_wallet: solv_protocol_wallet.key(),
        solv_manager: solv_manager.key(),

        vault_supported_token_mint: vault_supported_token_mint.key(),
        vault_receipt_token_mint: vault_receipt_token_mint.key(),
        solv_receipt_token_mint: solv_receipt_token_mint.key(),

        confirmed_vst_amount: vst_amount,
        deducted_vst_deposit_fee_amount,
        expected_srt_amount,
        one_srt_as_micro_vst,
    }))
}

pub fn process_complete_deposits(
    ctx: &mut Context<SolvManagerContext>,
    srt_amount: u64,
    new_one_srt_as_micro_vst: u64,
) -> Result<events::SolvManagerCompletedDeposits> {
    let SolvManagerContext {
        solv_manager,
        solv_protocol_wallet,
        vault_account,
        vault_receipt_token_mint,
        vault_supported_token_mint,
        solv_receipt_token_mint,
        vault_solv_receipt_token_account,
        ..
    } = ctx.accounts;

    let mut vault = vault_account.load_mut()?;

    let (old_one_srt_as_micro_vst, deducted_vst_extra_fee_amount, operation_reserved_srt_amount) =
        vault.offset_srt_receivables(srt_amount, new_one_srt_as_micro_vst, true)?;

    require_gte!(
        vault_solv_receipt_token_account.amount,
        vault.get_srt_total_reserved_amount(),
    );

    Ok(events::SolvManagerCompletedDeposits {
        vault: vault_account.key(),
        solv_protocol_wallet: solv_protocol_wallet.key(),
        solv_manager: solv_manager.key(),

        vault_supported_token_mint: vault_supported_token_mint.key(),
        vault_receipt_token_mint: vault_receipt_token_mint.key(),
        solv_receipt_token_mint: solv_receipt_token_mint.key(),

        received_srt_amount: srt_amount,
        operation_reserved_srt_amount,
        deducted_vst_extra_fee_amount,
        old_one_srt_as_micro_vst,
        new_one_srt_as_micro_vst,
    })
}

pub fn process_confirm_withdrawal_requests(
    ctx: &mut Context<SolvManagerContext>,
) -> Result<events::SolvManagerConfirmedWithdrawalRequests> {
    let SolvManagerContext {
        solv_manager,
        solv_protocol_wallet,
        vault_account,
        vault_receipt_token_mint,
        vault_supported_token_mint,
        solv_receipt_token_mint,
        vault_solv_receipt_token_account,
        solv_protocol_wallet_solv_receipt_token_account,
        token_program,
        ..
    } = ctx.accounts;

    let (confirmed_srt_amount, processing_vrt_amount, estimated_vst_amount) =
        vault_account.load_mut()?.confirm_withdrawal_requests()?;

    // TODO/phase3: CPI call to the Solv protocol - now just transfer
    if confirmed_srt_amount > 0 {
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
            confirmed_srt_amount,
            solv_receipt_token_mint.decimals,
        )?;
    }

    Ok(events::SolvManagerConfirmedWithdrawalRequests {
        vault: vault_account.key(),
        solv_protocol_wallet: solv_protocol_wallet.key(),
        solv_manager: solv_manager.key(),

        vault_supported_token_mint: vault_supported_token_mint.key(),
        vault_receipt_token_mint: vault_receipt_token_mint.key(),
        solv_receipt_token_mint: solv_receipt_token_mint.key(),

        confirmed_srt_amount,
        processing_vrt_amount,
        estimated_vst_amount,
    })
}

pub fn process_complete_withdrawal_requests(
    ctx: &mut Context<SolvManagerContext>,
    srt_amount: u64,
    vst_amount: u64,
    old_one_srt_as_micro_vst: u64,
) -> Result<events::SolvManagerCompletedWithdrawalRequests> {
    let SolvManagerContext {
        solv_manager,
        solv_protocol_wallet,
        vault_account,
        vault_receipt_token_mint,
        vault_supported_token_mint,
        solv_receipt_token_mint,
        vault_vault_supported_token_account,
        ..
    } = ctx.accounts;

    // TODO/phase3: CPI call to the solv protocol - now assumes that solv protocol has already sent VST to vault's ATA

    let mut vault = vault_account.load_mut()?;

    let (vst_reserved_amount_to_claim, vst_extra_amount_to_claim, vst_deducted_fee_amount) = vault
        .complete_withdrawal_requests(srt_amount, vst_amount, old_one_srt_as_micro_vst, true)?;

    require_gte!(
        vault_vault_supported_token_account.amount,
        vault.get_vst_total_reserved_amount(),
    );

    Ok(events::SolvManagerCompletedWithdrawalRequests {
        vault: vault_account.key(),
        solv_protocol_wallet: solv_protocol_wallet.key(),
        solv_manager: solv_manager.key(),

        vst_mint: vault_supported_token_mint.key(),
        vrt_mint: vault_receipt_token_mint.key(),
        srt_mint: solv_receipt_token_mint.key(),

        burnt_srt_amount: srt_amount,
        withdrawn_vst_amount: vst_amount,
        vst_reserved_amount_to_claim,
        vst_extra_amount_to_claim,
        vst_deducted_fee_amount,
    })
}

pub fn process_refresh_solv_receipt_token_redemption_rate(
    ctx: &mut Context<SolvManagerContext>,
    new_one_srt_as_micro_vst: u64,
) -> Result<events::SolvManagerRefreshedSrtRedemptionRate> {
    let SolvManagerContext {
        solv_manager,
        solv_protocol_wallet,
        vault_account,
        vault_supported_token_mint,
        vault_receipt_token_mint,
        solv_receipt_token_mint,
        ..
    } = ctx.accounts;

    let (
        old_one_srt_as_micro_vst,
        new_one_srt_as_micro_vst,
        old_one_vrt_as_micro_vst,
        new_one_vrt_as_micro_vst,
    ) = vault_account
        .load_mut()?
        .refresh_srt_exchange_rate_with_validation(new_one_srt_as_micro_vst, true)?;

    Ok(events::SolvManagerRefreshedSrtRedemptionRate {
        vault: vault_account.key(),
        solv_protocol_wallet: solv_protocol_wallet.key(),
        solv_manager: solv_manager.key(),

        vault_supported_token_mint: vault_supported_token_mint.key(),
        vault_receipt_token_mint: vault_receipt_token_mint.key(),
        solv_receipt_token_mint: solv_receipt_token_mint.key(),

        old_one_srt_as_micro_vst,
        new_one_srt_as_micro_vst,
        old_one_vrt_as_micro_vst,
        new_one_vrt_as_micro_vst,
    })
}

pub fn process_imply_solv_protocol_fee(
    ctx: &mut Context<SolvManagerContext>,
    new_one_srt_as_micro_vst: u64,
) -> Result<events::SolvManagerImpliedSolvProtocolFee> {
    let SolvManagerContext {
        solv_manager,
        solv_protocol_wallet,
        vault_account,
        vault_supported_token_mint,
        vault_receipt_token_mint,
        solv_receipt_token_mint,
        ..
    } = ctx.accounts;

    let (
        old_one_srt_as_micro_vst,
        new_one_srt_as_micro_vst,
        delta,
        vst_operation_receivable_amount,
    ) = vault_account
        .load_mut()?
        .adjust_srt_exchange_rate_with_extra_vst_receivables(new_one_srt_as_micro_vst, true)?;

    Ok(events::SolvManagerImpliedSolvProtocolFee {
        vault: vault_account.key(),
        solv_protocol_wallet: solv_protocol_wallet.key(),
        solv_manager: solv_manager.key(),

        vault_supported_token_mint: vault_supported_token_mint.key(),
        vault_receipt_token_mint: vault_receipt_token_mint.key(),
        solv_receipt_token_mint: solv_receipt_token_mint.key(),

        old_one_srt_as_micro_vst,
        new_one_srt_as_micro_vst,
        srt_operation_reserved_amount_as_vst_delta: delta,
        vst_operation_receivable_amount,
    })
}

pub fn process_confirm_donations(
    ctx: &mut Context<SolvManagerContext>,
    srt_amount: u64,
    vst_amount: u64,
) -> Result<events::SolvManagerConfirmedDonations> {
    let SolvManagerContext {
        solv_manager,
        solv_protocol_wallet,
        vault_account,
        vault_vault_supported_token_account,
        vault_solv_receipt_token_account,
        vault_supported_token_mint,
        vault_receipt_token_mint,
        solv_receipt_token_mint,
        ..
    } = ctx.accounts;

    let mut vault = vault_account.load_mut()?;

    let srt_amount = vault.donate_srt(srt_amount)?;
    let vst_amount = vault.donate_vst(vst_amount)?;

    require_gte!(
        vault_vault_supported_token_account.amount,
        vault.get_vst_total_reserved_amount(),
    );
    require_gte!(
        vault_solv_receipt_token_account.amount,
        vault.get_srt_total_reserved_amount(),
    );

    Ok(events::SolvManagerConfirmedDonations {
        vault: vault_account.key(),
        solv_protocol_wallet: solv_protocol_wallet.key(),
        solv_manager: solv_manager.key(),

        vault_supported_token_mint: vault_supported_token_mint.key(),
        vault_receipt_token_mint: vault_receipt_token_mint.key(),
        solv_receipt_token_mint: solv_receipt_token_mint.key(),

        donated_srt_amount: srt_amount,
        donated_vst_amount: vst_amount,
    })
}

// TODO/phase3: deprecate
#[event_cpi]
#[derive(Accounts)]
pub struct SolvManagerConfigurationContext<'info> {
    pub solv_manager: Signer<'info>,
    pub solv_protocol_wallet: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [VaultAccount::SEED, vault_receipt_token_mint.key().as_ref()],
        bump = vault_account.load()?.get_bump(),
        has_one = solv_manager @ VaultError::VaultAdminMismatchError,
        constraint = vault_account.load()?.is_latest_version() @ VaultError::InvalidAccountDataVersionError,
    )]
    pub vault_account: AccountLoader<'info, VaultAccount>,

    pub vault_receipt_token_mint: Account<'info, Mint>,
}

// TODO/phase3: deprecate
pub fn process_set_solv_protocol_wallet(
    ctx: &mut Context<SolvManagerConfigurationContext>,
) -> Result<()> {
    let SolvManagerConfigurationContext {
        vault_account,
        solv_protocol_wallet,
        ..
    } = ctx.accounts;

    vault_account
        .load_mut()?
        .set_solv_protocol_wallet(solv_protocol_wallet.key())?;

    Ok(())
}

// TODO/phase3: deprecate
pub fn process_set_solv_protocol_fee_rate(
    ctx: &mut Context<SolvManagerConfigurationContext>,
    deposit_fee_rate_bps: u16,
    withdrawal_fee_rate_bps: u16,
) -> Result<()> {
    let SolvManagerConfigurationContext { vault_account, .. } = ctx.accounts;

    vault_account
        .load_mut()?
        .set_solv_protocol_deposit_fee_rate_bps(deposit_fee_rate_bps)?
        .set_solv_protocol_withdrawal_fee_rate_bps(withdrawal_fee_rate_bps)?;

    Ok(())
}
