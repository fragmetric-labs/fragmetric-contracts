use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::constants::*;
use crate::modules::fund::FundAccount;
use crate::utils::{AccountLoaderExt, PDASeeds};

// will be used only once
#[event_cpi]
#[derive(Accounts)]
pub struct AdminFundAccountInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    /// Mint authority must be admin or fund account,
    /// otherwise `set_authority` CPI will fail.
    /// Therefore, no extra constraint is needed.
    #[account(mut)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub receipt_token_program: Program<'info, Token2022>,

    #[account(
        init,
        payer = payer,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump,
        space = std::cmp::min(
            solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE,
            8 + std::mem::size_of::<FundAccount>(),
        ),
    )]
    pub fund_account: AccountLoader<'info, FundAccount>,

    #[account(
        associated_token::mint = receipt_token_mint,
        associated_token::authority = fund_account,
        associated_token::token_program = receipt_token_program,
    )]
    pub fund_receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        seeds = [FundAccount::RESERVE_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub fund_reserve_account: SystemAccount<'info>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct AdminFundAccountUpdateContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.get_bump()?,
    )]
    pub fund_account: AccountLoader<'info, FundAccount>,

    #[account(
        seeds = [FundAccount::RESERVE_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub fund_reserve_account: SystemAccount<'info>,
}
