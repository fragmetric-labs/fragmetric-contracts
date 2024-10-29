use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::modules::normalize::NormalizedTokenAuthority;
use crate::utils::PDASeeds;

#[derive(Accounts)]
pub struct AdminNormalizedTokenAuthorityInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(address = ADMIN_PUBKEY)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub normalized_token_program: Program<'info, Token>,

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut)]
    pub normalized_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = payer,
        seeds = [NormalizedTokenAuthority::SEED, receipt_token_mint.key().as_ref(), normalized_token_mint.key().as_ref()],
        bump,
        space = 8 + NormalizedTokenAuthority::INIT_SPACE,
    )]
    pub normalized_token_authority: Account<'info, NormalizedTokenAuthority>,
}
