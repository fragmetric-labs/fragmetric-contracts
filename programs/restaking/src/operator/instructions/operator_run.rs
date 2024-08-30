use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

use crate::{common::*, constants::*, fund::*, operator::*};

#[derive(Accounts)]
pub struct OperatorRun<'info> {
    // Only the admin can run the operator manually.
    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

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

    #[account(mut, address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        token::mint = receipt_token_mint,
        token::authority = receipt_token_lock_authority,
        seeds = [RECEIPT_TOKEN_LOCK_ACCOUNT_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>, // fund's fragSOL lock account

    // TODO: use address lookup table!
    #[account(address = BSOL_STAKE_POOL_ADDRESS)]
    /// CHECK: will be checked and deserialized when needed
    pub token_pricing_source_0: UncheckedAccount<'info>,

    // TODO: use address lookup table!
    #[account(address = MSOL_STAKE_POOL_ADDRESS)]
    /// CHECK: will be checked and deserialized when needed
    pub token_pricing_source_1: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
}

impl<'info> OperatorRun<'info> {
    /// Manually run the operator.
    /// This instruction is only available to ADMIN
    pub fn operator_run(ctx: Context<Self>) -> Result<()> {
        let (receipt_token_price, receipt_token_total_supply) = Run::new(
            &mut ctx.accounts.fund,
            &mut ctx.accounts.receipt_token_lock_authority,
            &mut ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.receipt_token_lock_account,
            &[
                ctx.accounts.token_pricing_source_0.as_ref(),
                ctx.accounts.token_pricing_source_1.as_ref(),
            ],
            &ctx.accounts.token_program,
        )
        .run()?;

        emit!(OperatorRan {
            fund_info: FundInfo::new_from_fund(
                &ctx.accounts.fund,
                receipt_token_price,
                receipt_token_total_supply
            ),
        });

        Ok(())
    }
}
