use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

use crate::{constants::*, error::ErrorCode, fund::*, token::*, Empty};

#[derive(Accounts)]
pub struct FundRequestWithdrawal<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [USER_RECEIPT_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub user_receipt: Account<'info, UserReceipt>,

    #[account(
        mut,
        seeds = [FUND_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub fund: Box<Account<'info, Fund>>,

    #[account(
        mut,
        seeds = [FUND_TOKEN_AUTHORITY_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub fund_token_authority: Account<'info, Empty>,

    #[account(mut, address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        associated_token::mint = receipt_token_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>, // user's fragSOL token account
    #[account(
        mut,
        associated_token::mint = receipt_token_mint,
        associated_token::authority = fund_token_authority,
        associated_token::token_program = token_program,
    )]
    pub receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>, // fund's fragSOL lock account

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> FundRequestWithdrawal<'info> {
    pub fn request_withdrawal(mut ctx: Context<Self>, receipt_token_amount: u64) -> Result<()> {
        Self::lock_receipt_token(&mut ctx, receipt_token_amount)
            .map_err(|_| error!(ErrorCode::FundTokenTransferFailed))?;

        let withdrawal_request = ctx
            .accounts
            .fund
            .withdrawal_status
            .create_withdrawal_request(receipt_token_amount)?;
        let request_id = withdrawal_request.request_id;
        ctx.accounts
            .user_receipt
            .push_withdrawal_request(withdrawal_request)?;

        emit!(FundWithdrawalRequested {
            user: ctx.accounts.user.key(),
            user_lrt_account: ctx.accounts.receipt_token_account.key(),
            user_receipt: Clone::clone(&ctx.accounts.user_receipt),
            request_id,
            lrt_mint: ctx.accounts.receipt_token_mint.key(),
            lrt_requested_amount: receipt_token_amount,
            lrt_amount_in_user_lrt_account: ctx.accounts.receipt_token_account.amount,
        });

        Ok(())
    }

    fn lock_receipt_token(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        Self::call_burn_token_cpi(ctx, amount)?;
        Self::call_mint_token_cpi(ctx, amount)?;
        Self::call_transfer_hook(ctx, amount)
    }

    fn call_burn_token_cpi(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts.token_program.burn_token_cpi(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.receipt_token_account,
            ctx.accounts.user.to_account_info(),
            None,
            amount,
        )
    }

    fn call_mint_token_cpi(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        let bump = ctx.bumps.fund_token_authority;
        let key = ctx.accounts.receipt_token_mint.key();
        let signer_seeds = [FUND_TOKEN_AUTHORITY_SEED, key.as_ref(), &[bump]];

        ctx.accounts.token_program.mint_token_cpi(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.receipt_token_lock_account,
            ctx.accounts.fund_token_authority.to_account_info(),
            Some(&[signer_seeds.as_ref()]),
            amount,
        )
    }

    fn call_transfer_hook(ctx: &Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts.receipt_token_mint.transfer_hook(
            Some(&ctx.accounts.receipt_token_account),
            Some(&ctx.accounts.receipt_token_lock_account),
            &ctx.accounts.fund,
            amount,
        )
    }
}
