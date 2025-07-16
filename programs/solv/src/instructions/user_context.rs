use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{errors::VaultError, events, states::VaultAccount};

#[event_cpi]
#[derive(Accounts)]
pub struct UserContext<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [VaultAccount::SEED, vault_receipt_token_mint.key().as_ref()],
        bump = vault_account.load()?.get_bump(),
        has_one = solv_receipt_token_mint @ VaultError::SolvReceiptTokenMintMismatchError,
        constraint = vault_account.load()?.is_latest_version() @ VaultError::InvalidAccountDataVersionError,
    )]
    pub vault_account: AccountLoader<'info, VaultAccount>,

    pub solv_receipt_token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub vault_receipt_token_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = solv_receipt_token_mint,
        associated_token::authority = vault_account,
    )]
    pub vault_solv_receipt_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = solv_receipt_token_mint,
        associated_token::authority = user,
    )]
    pub user_solv_receipt_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = vault_receipt_token_mint,
        associated_token::authority = user,
    )]
    pub user_vault_receipt_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn process_deposit_solv_receipt_token(
    ctx: &mut Context<UserContext>,
    srt_amount: u64,
) -> Result<events::UserDepositedSRT> {
    let UserContext {
        user,
        vault_account,
        solv_receipt_token_mint,
        vault_receipt_token_mint,
        vault_solv_receipt_token_account,
        user_solv_receipt_token_account,
        user_vault_receipt_token_account,
        token_program,
        ..
    } = ctx.accounts;

    require_gt!(srt_amount, 0);
    require_gte!(user_solv_receipt_token_account.amount, srt_amount);

    let vrt_amount = vault_account.load_mut()?.mint_vrt_with_srt(srt_amount)?;

    anchor_spl::token::transfer_checked(
        CpiContext::new(
            token_program.to_account_info(),
            anchor_spl::token::TransferChecked {
                from: user_solv_receipt_token_account.to_account_info(),
                mint: solv_receipt_token_mint.to_account_info(),
                to: vault_solv_receipt_token_account.to_account_info(),
                authority: user.to_account_info(),
            },
        ),
        srt_amount,
        solv_receipt_token_mint.decimals,
    )?;

    if vrt_amount > 0 {
        anchor_spl::token::mint_to(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                anchor_spl::token::MintTo {
                    mint: vault_receipt_token_mint.to_account_info(),
                    to: user_vault_receipt_token_account.to_account_info(),
                    authority: vault_account.to_account_info(),
                },
                &[&vault_account.load()?.get_seeds()],
            ),
            vrt_amount,
        )?;
    }

    Ok(events::UserDepositedSRT {
        vault: vault_account.key(),
        vault_receipt_token_mint: vault_receipt_token_mint.key(),
        user: user.key(),
        user_vrt_account: user_vault_receipt_token_account.key(),
        deposited_srt_amount: srt_amount,
        minted_vrt_amount: vrt_amount,
    })
}
