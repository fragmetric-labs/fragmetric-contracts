use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

use crate::constants::*;
use crate::modules::common::PDASignerSeeds;
use crate::modules::fund::{FundAccount, ReceiptTokenLockAuthority};

// will be used only once
#[derive(Accounts)]
pub struct AdminFundReceiptTokenLockAuthorityInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = payer,
        seeds = [ReceiptTokenLockAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump,
        space = 8 + ReceiptTokenLockAuthority::INIT_SPACE,
    )]
    pub receipt_token_lock_authority: Account<'info, ReceiptTokenLockAuthority>,
}

impl<'info> AdminFundReceiptTokenLockAuthorityInitialContext<'info> {
    pub fn initialize_receipt_token_lock_authority(
        ctx: Context<AdminFundReceiptTokenLockAuthorityInitialContext>,
    ) -> Result<()> {
        ctx.accounts
            .receipt_token_lock_authority
            .initialize_if_needed(
                ctx.bumps.receipt_token_lock_authority,
                ctx.accounts.receipt_token_mint.key(),
            );
        Ok(())
    }
}

// will be used only once
#[derive(Accounts)]
pub struct AdminFundReceiptTokenLockAccountInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub receipt_token_program: Program<'info, Token2022>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [ReceiptTokenLockAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump = receipt_token_lock_authority.bump,
    )]
    pub receipt_token_lock_authority: Account<'info, ReceiptTokenLockAuthority>,

    #[account(
        init,
        payer = payer,
        token::mint = receipt_token_mint,
        token::authority = receipt_token_lock_authority,
        token::token_program = receipt_token_program,
        seeds = [ReceiptTokenLockAuthority::TOKEN_ACCOUNT_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>,
}

impl<'info> AdminFundReceiptTokenLockAccountInitialContext<'info> {
    pub fn initialize_receipt_token_lock_account(_ctx: Context<Self>) -> Result<()> {
        Ok(())
    }
}

// will be used only once
#[derive(Accounts)]
pub struct AdminFundAccountInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = payer,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump,
        space = 8 + FundAccount::INIT_SPACE,
    )]
    pub fund_account: Box<Account<'info, FundAccount>>,
}

impl<'info> AdminFundAccountInitialContext<'info> {
    pub fn initialize_fund_account(ctx: Context<Self>) -> Result<()> {
        let receipt_token_mint_key = ctx.accounts.receipt_token_mint.key();

        ctx.accounts
            .fund_account
            .initialize_if_needed(ctx.bumps.fund_account, receipt_token_mint_key);

        Ok(())
    }
}

#[derive(Accounts)]
pub struct AdminFundContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    #[account(mut, address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub receipt_token_program: Program<'info, Token2022>,

    #[account(
        seeds = [ReceiptTokenLockAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump = receipt_token_lock_authority.bump,
        has_one = receipt_token_mint,
    )]
    pub receipt_token_lock_authority: Account<'info, ReceiptTokenLockAuthority>,

    #[account(
        mut,
        token::mint = receipt_token_mint,
        token::authority = receipt_token_lock_authority,
        token::token_program = receipt_token_program,
        seeds = [ReceiptTokenLockAuthority::TOKEN_ACCOUNT_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>, // fund's fragSOL lock account

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.bump,
        has_one = receipt_token_mint,
    )]
    pub fund_account: Box<Account<'info, FundAccount>>,
}

impl<'info> AdminFundContext<'info> {
    pub fn update_fund_account(ctx: Context<Self>) -> Result<()> {
        let bump = ctx.accounts.fund_account.bump;
        ctx.accounts
            .fund_account
            .initialize_if_needed(bump, ctx.accounts.receipt_token_mint.key());
        Ok(())
    }
}
