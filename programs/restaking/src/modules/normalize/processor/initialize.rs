use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, spl_token, Token},
    token_interface::{Mint, TokenAccount},
};

use crate::modules::normalize::*;

pub fn process_initialize_normalized_token_pool_account<'info>(
    admin: &Signer<'info>,
    normalized_token_mint: &InterfaceAccount<'info, Mint>,
    normalized_token_account: &InterfaceAccount<'info, TokenAccount>,
    normalized_token_pool_account: &mut Account<NormalizedTokenPoolAccount>,
    normalized_token_program: &Program<'info, Token>,
    bump: u8,
) -> Result<()> {
    normalized_token_pool_account.initialize(
        bump,
        normalized_token_mint.key(),
        normalized_token_program.key(),
        normalized_token_account.key(),
    );

    token::set_authority(
        CpiContext::new(
            normalized_token_program.to_account_info(),
            token::SetAuthority {
                current_authority: admin.to_account_info(),
                account_or_mint: normalized_token_mint.to_account_info(),
            },
        ),
        spl_token::instruction::AuthorityType::MintTokens,
        Some(normalized_token_pool_account.key()),
    )?;

    Ok(())
}
