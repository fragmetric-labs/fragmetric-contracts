use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use fragmetric_util::{request, Upgradable};

use crate::{constants::*, error::ErrorCode, fund::*};

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
        seeds = [RECEIPT_TOKEN_AUTHORITY_SEED, receipt_token_mint.key().as_ref()],
        bump,
        realloc = 8 + ReceiptTokenAuthority::INIT_SPACE,
        // TODO must paid by fund
        realloc::payer = user,
        realloc::zero = false,
    )]
    pub receipt_token_authority: Account<'info, ReceiptTokenAuthority>,
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = receipt_token_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>, // user's fragSOL token account

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> FundDepositSOL<'info> {
    pub fn deposit_sol(ctx: Context<Self>, request: FundDepositSOLRequest) -> Result<()> {
        let FundDepositSOLArgs { amount } = request.into();
        let receipt_token_account = ctx.accounts.receipt_token_account.key();
        msg!("receipt_token_account: {}", receipt_token_account);

        Self::transfer_sol_cpi(&ctx, amount)?;
        ctx.accounts.fund.to_latest_version().deposit_sol(amount)
        // TODO mint receipt token
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
