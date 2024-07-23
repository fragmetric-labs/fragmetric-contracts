use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked},
};

use crate::fund::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct DepositToken<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    #[account(
        mut,
        seeds = [FUND_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub fund: Box<Account<'info, Fund>>,

    #[account(
        mut,
        seeds = [RECEIPT_TOKEN_AUTHORITY_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub receipt_token_authority: Box<Account<'info, ReceiptTokenAuthority>>,

    #[account(mut)]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>, // lst token mint account
    #[account(mut, token::mint = token_mint, token::authority = depositor.key())]
    pub depositor_token_account: Box<InterfaceAccount<'info, TokenAccount>>, // depositor's lst token account
    #[account(
        init_if_needed,
        payer = depositor,
        seeds = [FUND_SEED, token_mint.key().as_ref()],
        bump,
        token::mint = token_mint,
        token::authority = receipt_token_authority,
    )]
    pub fund_token_account: Box<InterfaceAccount<'info, TokenAccount>>, // fund's lst token account

    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = depositor,
        associated_token::mint = receipt_token_mint,
        associated_token::authority = depositor,
        associated_token::token_program = token_program,
    )]
    pub receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>, // user's fragSOL token account

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> DepositToken<'info> {
    pub fn handler(ctx: Context<Self>, amount: u64) -> Result<()> {
        Self::transfer_token_cpi_ctx(&ctx, amount)?;
    
        let fund = &mut ctx.accounts.fund;
        let token = &ctx.accounts.token_mint;
        Ok(fund.deposit_token(token.key(), amount)?)
    }

    pub fn transfer_token_cpi_ctx(ctx: &Context<Self>, amount: u64) -> Result<()> {
        let depositor_token_account = &ctx.accounts.depositor_token_account; // from depositor
        let fund_token_account = &ctx.accounts.fund_token_account; // to fund
        let token_mint = &ctx.accounts.token_mint;
        let token_program = &ctx.accounts.token_program;
        let authority = &ctx.accounts.depositor;

        let token_transfer_cpi_ctx = CpiContext::new(
            token_program.to_account_info(),
            TransferChecked {
                from: depositor_token_account.to_account_info(),
                to: fund_token_account.to_account_info(),
                mint: token_mint.to_account_info(),
                authority: authority.to_account_info(),
            },
        );

        Ok(transfer_checked(token_transfer_cpi_ctx, amount, token_mint.decimals)?)
    }
}
