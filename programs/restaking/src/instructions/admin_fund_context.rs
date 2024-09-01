use anchor_lang::prelude::*;
use anchor_spl::{token_2022::Token2022, token_interface::{Mint, TokenAccount}};

use crate::constants::*;
use crate::modules::common::PDASignerSeeds;
use crate::modules::fund::{FundAccount, ReceiptTokenLockAuthority};

#[derive(Accounts)]
pub struct AdminFundContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub receipt_token_program: Program<'info, Token2022>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = payer,
        seeds = [ReceiptTokenLockAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump,
        space = 8 + ReceiptTokenLockAuthority::INIT_SPACE,
    )]
    pub receipt_token_lock_authority: Account<'info, ReceiptTokenLockAuthority>,

    #[account(
        init_if_needed,
        payer = payer,
        token::mint = receipt_token_mint,
        token::authority = receipt_token_lock_authority,
        token::token_program = receipt_token_program,
        seeds = [ReceiptTokenLockAuthority::TOKEN_ACCOUNT_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()], // fund + <any receipt token mint account>
        bump,
        space = 8 + FundAccount::INIT_SPACE,
    )]
    pub fund_account: Box<Account<'info, FundAccount>>,
}

impl<'info> AdminFundContext<'info> {
    pub fn initialize_fund_accounts_if_needed(ctx: Context<Self>) -> Result<()> {
        let receipt_token_mint_key = ctx.accounts.receipt_token_mint.key();

        ctx.accounts
            .fund_account
            .initialize_if_needed(
                ctx.bumps.fund_account,
                receipt_token_mint_key,
            );
        ctx.accounts
            .receipt_token_lock_authority
            .initialize_if_needed(
                ctx.bumps.receipt_token_lock_authority,
                receipt_token_mint_key,
            );

        Ok(())
    }
}
