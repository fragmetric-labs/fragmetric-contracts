use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::modules::normalization::NormalizedTokenPoolAccount;
use crate::utils::PDASeeds;

#[derive(Accounts)]
pub struct AdminNormalizedTokenPoolInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub normalized_token_program: Program<'info, Token>,

    #[account(
        mut,
        address = NSOL_MINT_ADDRESS,
        constraint = normalized_token_mint.supply == 0,
    )]
    pub normalized_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = payer,
        space = 8 + NormalizedTokenPoolAccount::INIT_SPACE,
        seeds = [NormalizedTokenPoolAccount::SEED, normalized_token_mint.key().as_ref()],
        bump,
    )]
    pub normalized_token_pool_account: Box<Account<'info, NormalizedTokenPoolAccount>>,
}
