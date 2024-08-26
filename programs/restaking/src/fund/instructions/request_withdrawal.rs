use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

use crate::{common::*, constants::*, error::ErrorCode, fund::*, token::*};

#[derive(Accounts)]
pub struct FundRequestWithdrawal<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserReceipt::INIT_SPACE,
        seeds = [UserReceipt::SEED, user.key().as_ref(), receipt_token_mint.key().as_ref()],
        bump,
        constraint = user_receipt.data_version == 0 || user_receipt.user == user.key(),
        constraint = user_receipt.data_version == 0 || user_receipt.receipt_token_mint == receipt_token_mint.key(),
    )]
    pub user_receipt: Account<'info, UserReceipt>,

    #[account(
        mut,
        seeds = [Fund::SEED, receipt_token_mint.key().as_ref()],
        bump = fund.bump,
        has_one = receipt_token_mint,
    )]
    pub fund: Box<Account<'info, Fund>>,

    #[account(
        seeds = [ReceiptTokenLockAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump = receipt_token_lock_authority.bump,
        has_one = receipt_token_mint,
    )]
    pub receipt_token_lock_authority: Account<'info, ReceiptTokenLockAuthority>,

    #[account(
        seeds = [ReceiptTokenMintAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump = receipt_token_mint_authority.bump,
        has_one = receipt_token_mint,
    )]
    pub receipt_token_mint_authority: Account<'info, ReceiptTokenMintAuthority>,

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
        associated_token::authority = receipt_token_lock_authority,
        associated_token::token_program = token_program,
    )]
    pub receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>, // fund's fragSOL lock account

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> FundRequestWithdrawal<'info> {
    pub fn request_withdrawal(mut ctx: Context<Self>, receipt_token_amount: u64) -> Result<()> {
        // Initialize
        ctx.accounts.user_receipt.initialize_if_needed(
            ctx.bumps.user_receipt,
            ctx.accounts.user.key(),
            ctx.accounts.receipt_token_mint.key(),
        );

        // Verify
        require_gte!(
            ctx.accounts.receipt_token_account.amount,
            receipt_token_amount
        );

        // Step 1: Create withdrawal request
        ctx.accounts
            .fund
            .withdrawal_status
            .check_withdrawal_enabled()?;
        let withdrawal_request = ctx
            .accounts
            .fund
            .withdrawal_status
            .create_withdrawal_request(receipt_token_amount)?;
        let request_id = withdrawal_request.request_id;
        ctx.accounts
            .user_receipt
            .push_withdrawal_request(withdrawal_request)?;

        // Step 2: Lock receipt token
        Self::call_burn_token_cpi(&mut ctx, receipt_token_amount)?;
        Self::call_mint_token_cpi(&mut ctx, receipt_token_amount)?;
        Self::call_transfer_hook(&ctx, receipt_token_amount)?;

        emit!(FundWithdrawalRequested {
            user: ctx.accounts.user.key(),
            user_receipt_token_account: ctx.accounts.receipt_token_account.key(),
            user_receipt: Clone::clone(&ctx.accounts.user_receipt),
            request_id,
            receipt_token_mint: ctx.accounts.receipt_token_mint.key(),
            receipt_token_requested_amount: receipt_token_amount,
            receipt_token_amount_in_user_receipt_token_account: ctx
                .accounts
                .receipt_token_account
                .amount,
        });

        Ok(())
    }

    fn call_burn_token_cpi(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts
            .token_program
            .burn_token_cpi(
                &mut ctx.accounts.receipt_token_mint,
                &mut ctx.accounts.receipt_token_account,
                ctx.accounts.user.to_account_info(),
                None,
                amount,
            )
            .map_err(|_| error!(ErrorCode::FundTokenTransferFailed))
    }

    fn call_mint_token_cpi(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts
            .token_program
            .mint_token_cpi(
                &mut ctx.accounts.receipt_token_mint,
                &mut ctx.accounts.receipt_token_lock_account,
                ctx.accounts.receipt_token_mint_authority.to_account_info(),
                Some(&[ctx
                    .accounts
                    .receipt_token_mint_authority
                    .signer_seeds()
                    .as_ref()]),
                amount,
            )
            .map_err(|_| error!(ErrorCode::FundTokenTransferFailed))
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
