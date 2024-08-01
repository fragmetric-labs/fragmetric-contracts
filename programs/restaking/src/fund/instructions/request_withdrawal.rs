use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{burn, mint_to, Burn, Mint, MintTo, TokenAccount},
};
use fragmetric_util::{request, Upgradable};

use crate::{constants::*, error::ErrorCode, fund::*, Empty};

#[derive(Accounts)]
pub struct FundRequestWithdrawal<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [USER_ACCOUNT_SEED, receipt_token_mint.key().as_ref()],
        bump,
        space = 8 + UserAccount::INIT_SPACE,
    )]
    pub user_account: Account<'info, UserAccount>,

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
    #[account(
        mut,
        associated_token::mint = receipt_token_mint,
        associated_token::authority = fund_token_authority,
        associated_token::token_program = token_program,
    )]
    pub receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>, // fund's fragSOL lock account

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> FundRequestWithdrawal<'info> {
    pub fn request_withdrawal(
        ctx: Context<Self>,
        request: FundRequestWithdrawalRequest,
    ) -> Result<()> {
        let FundRequestWithdrawalArgs {
            receipt_token_amount,
        } = request.into();
        Self::burn_token_cpi(&ctx, receipt_token_amount)?;
        Self::mint_token_cpi(&ctx, receipt_token_amount)?;

        let Self {
            fund, user_account, ..
        } = ctx.accounts;
        let withdrawal_request = fund
            .to_latest_version()
            .create_withdrawal_request(receipt_token_amount)?;
        user_account
            .to_latest_version()
            .push_withdrawal_request(withdrawal_request)
    }

    fn burn_token_cpi(ctx: &Context<Self>, amount: u64) -> Result<()> {
        let Self {
            user,
            receipt_token_mint,
            receipt_token_account,
            token_program,
            ..
        } = &*ctx.accounts;

        let burn_token_cpi_ctx = CpiContext::new(
            token_program.to_account_info(),
            Burn {
                mint: receipt_token_mint.to_account_info(),
                from: receipt_token_account.to_account_info(),
                authority: user.to_account_info(),
            },
        );

        burn(burn_token_cpi_ctx, amount).map_err(|_| error!(ErrorCode::FundReceiptTokenLockFailed))
    }

    fn mint_token_cpi(ctx: &Context<Self>, amount: u64) -> Result<()> {
        let Self {
            receipt_token_lock_account,
            receipt_token_mint,
            fund_token_authority,
            token_program,
            ..
        } = &*ctx.accounts;

        let bump = ctx.bumps.fund_token_authority;
        let receipt_token_mint_key = receipt_token_mint.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            FUND_TOKEN_AUTHORITY_SEED,
            receipt_token_mint_key.as_ref(),
            &[bump],
        ]];

        let mint_token_cpi_ctx = CpiContext::new_with_signer(
            token_program.to_account_info(),
            MintTo {
                mint: receipt_token_mint.to_account_info(),
                to: receipt_token_lock_account.to_account_info(),
                authority: fund_token_authority.to_account_info(),
            },
            signer_seeds,
        );

        mint_to(mint_token_cpi_ctx, amount)
            .map_err(|_| error!(ErrorCode::FundReceiptTokenLockFailed))
    }
}

pub struct FundRequestWithdrawalArgs {
    pub receipt_token_amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
#[request(FundRequestWithdrawalArgs)]
pub enum FundRequestWithdrawalRequest {
    V1(FundRequestWithdrawalV1),
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FundRequestWithdrawalV1 {
    pub receipt_token_amount: u64,
}

impl From<FundRequestWithdrawalRequest> for FundRequestWithdrawalArgs {
    fn from(value: FundRequestWithdrawalRequest) -> Self {
        match value {
            FundRequestWithdrawalRequest::V1(value) => Self {
                receipt_token_amount: value.receipt_token_amount,
            },
        }
    }
}
