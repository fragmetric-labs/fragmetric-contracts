use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

use crate::{common::*, constants::*, error::ErrorCode, fund::*, token::*};

#[derive(Accounts)]
pub struct FundDepositSOL<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [USER_RECEIPT_SEED, receipt_token_mint.key().as_ref()],
        bump,
        space = 8 + UserReceipt::INIT_SPACE,
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
        init_if_needed,
        payer = user,
        associated_token::mint = receipt_token_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>, // user's fragSOL token account

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> FundDepositSOL<'info> {
    pub fn deposit_sol(mut ctx: Context<Self>, amount: u64) -> Result<()> {
        let receipt_token_account = ctx.accounts.receipt_token_account.key();
        msg!("receipt_token_account: {}", receipt_token_account);

        Self::transfer_sol_cpi(&ctx, amount)?;
        ctx.accounts.fund.deposit_sol(amount)?;

        let mint_amount = Self::get_receipt_token_by_sol_exchange_rate(&ctx, amount)?;
        Self::mint_receipt_token(&mut ctx, mint_amount)?;

        emit!(FundSOLDeposited {
            user: ctx.accounts.user.key(),
            user_lrt_account: ctx.accounts.receipt_token_account.key(),
            user_receipt: Clone::clone(&ctx.accounts.user_receipt),
            sol_deposit_amount: amount,
            sol_amount_in_fund: ctx.accounts.fund.sol_amount_in,
            minted_lrt_mint: ctx.accounts.receipt_token_mint.key(),
            minted_lrt_amount: mint_amount,
            lrt_amount_in_user_lrt_account: ctx.accounts.receipt_token_account.amount,
            wallet_provider: None,
            fpoint_accrual_rate_multiplier: None,
            fund_info: FundInfo::new_from_fund(ctx.accounts.fund.as_ref()),
        });

        Ok(())
    }

    fn transfer_sol_cpi(ctx: &Context<Self>, amount: u64) -> Result<()> {
        let Self {
            user,
            fund,
            system_program,
            ..
        } = &*ctx.accounts;

        let sol_transfer_cpi_ctx = CpiContext::new(
            system_program.to_account_info(),
            system_program::Transfer {
                from: user.to_account_info(),
                to: fund.to_account_info(),
            },
        );

        msg!("Transferring from {} to {}", user.key, fund.key());

        system_program::transfer(sol_transfer_cpi_ctx, amount)
            .map_err(|_| error!(ErrorCode::FundSOLTransferFailed))?;

        msg!("Transferred {} SOL", amount);

        Ok(())
    }

    #[allow(unused_variables)]
    fn get_receipt_token_by_sol_exchange_rate(ctx: &Context<Self>, amount: u64) -> Result<u64> {
        Ok(amount)
    }

    fn mint_receipt_token(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        let receipt_token_account_key = ctx.accounts.receipt_token_account.key();
        msg!(
            "user's receipt token account key: {:?}",
            receipt_token_account_key
        );

        Self::call_mint_token_cpi(ctx, amount)?;
        msg!(
            "Minted {} to user token account {:?}",
            amount,
            receipt_token_account_key
        );

        Self::call_transfer_hook(ctx, amount)?;

        Ok(())
    }

    fn call_mint_token_cpi(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        let bump = ctx.bumps.fund_token_authority;
        let key = ctx.accounts.receipt_token_mint.key();
        let signer_seeds = [FUND_TOKEN_AUTHORITY_SEED, key.as_ref(), &[bump]];

        ctx.accounts.token_program.mint_token_cpi(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.receipt_token_account,
            ctx.accounts.fund_token_authority.to_account_info(),
            Some(&[signer_seeds.as_ref()]),
            amount,
        )
    }

    fn call_transfer_hook(ctx: &Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts.receipt_token_mint.transfer_hook(
            None,
            Some(&ctx.accounts.receipt_token_account),
            &ctx.accounts.fund,
            amount,
        )
    }
}
