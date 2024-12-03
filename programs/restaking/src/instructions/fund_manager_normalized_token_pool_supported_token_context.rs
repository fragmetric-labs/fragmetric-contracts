use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::*;
use crate::modules::normalization::NormalizedTokenPoolAccount;
use crate::utils::PDASeeds;

#[derive(Accounts)]
pub struct FundManagerNormalizedTokenPoolSupportedTokenContext<'info> {
    #[account(address = FUND_MANAGER_PUBKEY)]
    pub fund_manager: Signer<'info>,

    #[account(address = FRAGSOL_NORMALIZED_TOKEN_MINT_ADDRESS)]
    pub normalized_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [NormalizedTokenPoolAccount::SEED, normalized_token_mint.key().as_ref()],
        bump = normalized_token_pool_account.get_bump(),
        has_one = normalized_token_mint,
    )]
    pub normalized_token_pool_account: Box<Account<'info, NormalizedTokenPoolAccount>>,

    pub normalized_token_program: Program<'info, Token>,

    pub supported_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub supported_token_program: Interface<'info, TokenInterface>,

    #[account(
        associated_token::mint = supported_token_mint,
        associated_token::authority = normalized_token_pool_account,
        associated_token::token_program = supported_token_program,
    )]
    pub normalized_token_pool_supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
}
