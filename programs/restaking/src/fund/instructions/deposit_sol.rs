use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{mint_to, MintTo, Token2022},
    token_interface::{Mint, TokenAccount},
};
use fragmetric_util::{request, Upgradable};

use crate::{constants::*, error::ErrorCode, fund::*, Empty, token::TokenTransferHook};

#[derive(Accounts)]
pub struct FundDepositSOL<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [FUND_SEED, receipt_token_mint.key().as_ref()],
        bump,
        realloc = 8 + Fund::INIT_SPACE,
        // TODO must paid by fund
        realloc::payer = user,
        realloc::zero = false,
    )]
    pub fund: Account<'info, Fund>,

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
    pub fn deposit_sol(ctx: Context<Self>, request: FundDepositSOLRequest) -> Result<()> {
        let FundDepositSOLArgs { amount } = request.into();
        let receipt_token_account = ctx.accounts.receipt_token_account.key();
        msg!("receipt_token_account: {}", receipt_token_account);

        Self::transfer_sol_cpi(&ctx, amount)?;
        ctx.accounts.fund.to_latest_version().deposit_sol(amount)?;

        let mint_amount = Self::get_receipt_token_by_sol_exchange_rate(&ctx, amount)?;
        Self::mint_receipt_token(ctx, mint_amount)
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
            .map_err(|_| ErrorCode::FundSOLTransferFailed)?;

        msg!("Transferred {} SOL", amount);

        Ok(())
    }

    fn get_receipt_token_by_sol_exchange_rate(ctx: &Context<Self>, amount: u64) -> Result<u64> {
        Ok(amount)
    }

    fn mint_receipt_token(ctx: Context<Self>, amount: u64) -> Result<()> {
        let receipt_token_account_key = ctx.accounts.receipt_token_account.key();
        msg!("user's receipt token account key: {:?}", receipt_token_account_key);

        Self::mint_token_cpi(&ctx, amount)?;
        msg!("Minted {} to user token account {:?}", amount, receipt_token_account_key);

        Self::call_transfer_hook(&ctx, amount)?;

        Ok(())
    }

    fn mint_token_cpi(ctx: &Context<Self>, amount: u64) -> Result<()> {
        let bump = ctx.bumps.fund_token_authority;
        // PDA signer seeds
        let receipt_token_mint_key = ctx.accounts.receipt_token_mint.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            FUND_TOKEN_AUTHORITY_SEED,
            receipt_token_mint_key.as_ref(),
            &[bump],
        ]];

        let mint_token_cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.receipt_token_mint.to_account_info(),
                to: ctx.accounts.receipt_token_account.to_account_info(),
                authority: ctx.accounts.fund_token_authority.to_account_info(),
            },
        ).with_signer(signer_seeds);

        mint_to(mint_token_cpi_ctx, amount)
    }

    fn call_transfer_hook(ctx: &Context<Self>, amount: u64) -> Result<()> {
        TokenTransferHook::call_transfer_hook(
            ctx.accounts.receipt_token_mint.to_account_info(),
            ctx.accounts.receipt_token_mint.to_account_info(),
            ctx.accounts.receipt_token_account.to_account_info(),
            ctx.accounts.user.to_account_info(),
            amount,
        )
    }
}

pub struct FundDepositSOLArgs {
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
#[request(FundDepositSOLArgs)]
pub enum FundDepositSOLRequest {
    V1(FundDepositSOLRequestV1),
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FundDepositSOLRequestV1 {
    pub amount: u64,
}

impl From<FundDepositSOLRequest> for FundDepositSOLArgs {
    fn from(value: FundDepositSOLRequest) -> Self {
        match value {
            FundDepositSOLRequest::V1(value) => Self {
                amount: value.amount,
            },
        }
    }
}
