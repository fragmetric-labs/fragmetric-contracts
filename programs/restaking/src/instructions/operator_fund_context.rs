use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::modules::fund::FundAccount;
use crate::utils::PDASeeds;

// TODO: deprecate
#[derive(Accounts)]
pub struct OperatorFundContext<'info> {
    pub operator: Signer<'info>,

    #[account(mut, address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub receipt_token_program: Program<'info, Token2022>,

    #[account(
        mut,
        associated_token::mint = receipt_token_mint,
        associated_token::authority = fund_account,
        associated_token::token_program = receipt_token_program,
    )]
    pub receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>, // fund's fragSOL lock account

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.get_bump(),
        has_one = receipt_token_mint,
        constraint = fund_account.is_latest_version() @ ErrorCode::InvalidDataVersionError,
    )]
    pub fund_account: Box<Account<'info, FundAccount>>,
}

#[derive(Accounts)]
pub struct OperatorFundContext2<'info> {
    #[account(mut)]
    pub operator: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(mut, address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.get_bump(),
        has_one = receipt_token_mint,
        constraint = fund_account.is_latest_version() @ ErrorCode::InvalidDataVersionError,
    )]
    pub fund_account: Box<Account<'info, FundAccount>>,
}
