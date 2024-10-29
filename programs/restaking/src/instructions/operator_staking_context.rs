use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::*;
use crate::modules::fund::*;
use crate::utils::PDASeeds;

#[derive(Accounts)]
pub struct OperatorMoveFundToOperationReserveAccountContext<'info> {
    pub operator: Signer<'info>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.get_bump(),
    )]
    pub fund_account: Box<Account<'info, FundAccount>>,

    #[account(
        mut,
        seeds = [FundAccount::OPERATION_RESERVED_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub operation_reserve_account: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct OperatorStakingContext<'info> {
    pub operator: Signer<'info>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [FundAccount::SEED, receipt_token_mint.key().as_ref()],
        bump = fund_account.get_bump(),
    )]
    pub fund_account: Box<Account<'info, FundAccount>>,

    #[account(
        mut,
        seeds = [FundAccount::OPERATION_RESERVED_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub operation_reserve_account: SystemAccount<'info>,

    #[account(
        seeds = [SupportedTokenAuthority::SEED, receipt_token_mint.key().as_ref(), spl_pool_token_mint.key().as_ref()],
        bump = supported_token_authority.get_bump(),
        has_one = receipt_token_mint,
        constraint = supported_token_authority.supported_token_mint == spl_pool_token_mint.key(),
    )]
    pub supported_token_authority: Box<Account<'info, SupportedTokenAuthority>>,

    #[account(mut)]
    pub spl_pool_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        token::mint = spl_pool_token_mint,
        token::authority = supported_token_authority,
        token::token_program = supported_token_program,
        seeds = [SupportedTokenAuthority::TOKEN_ACCOUNT_SEED, receipt_token_mint.key().as_ref(), spl_pool_token_mint.key().as_ref()],
        bump,
    )]
    pub supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub supported_token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}
