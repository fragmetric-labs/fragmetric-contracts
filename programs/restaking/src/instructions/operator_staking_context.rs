use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::*;
use crate::modules::fund::*;
use crate::utils::PDASeeds;

// TODO v0.3/operation/staking: deprecate
#[derive(Accounts)]
pub struct OperatorStakingContext<'info> {
    pub operator: Signer<'info>,

    /// CHECK
    pub spl_stake_pool_program: AccountInfo<'info>,

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
        seeds = [FundAccount::RESERVE_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub fund_reserve_account: SystemAccount<'info>,

    #[account(mut)]
    pub spl_pool_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = spl_pool_token_mint,
        associated_token::authority = fund_account,
        associated_token::token_program = supported_token_program,
    )]
    pub fund_supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub supported_token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
