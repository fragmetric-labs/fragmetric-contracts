use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::*;
use crate::modules::normalization::{NormalizedTokenPoolAccount, NormalizedTokenWithdrawalAccount};
use crate::utils::PDASeeds;

#[event_cpi]
#[derive(Accounts)]
pub struct OperatorNormalizedTokenPoolContext<'info> {
    pub operator: Signer<'info>,

    #[account(mut)]
    pub normalized_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [NormalizedTokenPoolAccount::SEED, normalized_token_mint.key().as_ref()],
        bump = normalized_token_pool_account.get_bump(),
        has_one = normalized_token_mint,
    )]
    pub normalized_token_pool_account: Box<Account<'info, NormalizedTokenPoolAccount>>,

    pub normalized_token_program: Program<'info, Token>,
}
