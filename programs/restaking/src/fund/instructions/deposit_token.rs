use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{constants::*, error::ErrorCode, fund::*};

#[derive(Accounts)]
pub struct FundDepositToken<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

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
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = receipt_token_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>, // user's fragSOL token account

    #[account(mut)]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>, // lst token mint account
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = user.key()
    )]
    pub user_token_account: Box<InterfaceAccount<'info, TokenAccount>>, // depositor's lst token account
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token_mint,
        associated_token::authority = receipt_token_authority,
        associated_token::token_program = token_program,
    )]
    pub fund_token_account: Box<InterfaceAccount<'info, TokenAccount>>, // fund's lst token account

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> FundDepositToken<'info> {
    pub fn deposit_token(ctx: Context<Self>, request: FundDepositTokenRequest) -> Result<()> {
        let amount = request.amount;
        Self::transfer_token_cpi(&ctx, amount)?;

        let Self {
            fund, token_mint, ..
        } = ctx.accounts;
        fund.deposit_token(token_mint.key(), amount)
        // TODO mint receipt token
    }

    fn transfer_token_cpi(ctx: &Context<Self>, amount: u64) -> Result<()> {
        let Self {
            user: authority,
            user_token_account,
            fund_token_account,
            token_mint,
            token_program,
            ..
        } = &*ctx.accounts;

        let token_transfer_cpi_ctx = CpiContext::new(
            token_program.to_account_info(),
            TransferChecked {
                from: user_token_account.to_account_info(),
                to: fund_token_account.to_account_info(),
                mint: token_mint.to_account_info(),
                authority: authority.to_account_info(),
            },
        );

        transfer_checked(token_transfer_cpi_ctx, amount, token_mint.decimals)
            .map_err(|_| ErrorCode::FundTokenTransferFailed)?;

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FundDepositTokenRequest {
    pub amount: u64,
}
