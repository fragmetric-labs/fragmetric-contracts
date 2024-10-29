use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, spl_token, Token},
    token_interface::{Mint, TokenAccount},
};

use crate::modules::normalize::*;

pub fn process_initialize_normalized_token_authority<'info>(
    admin: &Signer<'info>,
    receipt_token_mint: &InterfaceAccount<Mint>,
    normalized_token_mint: &InterfaceAccount<'info, Mint>,
    normalized_token_authority: &mut Account<NormalizedTokenAuthority>,
    normalized_token_program: &Program<'info, Token>,
    bump: u8,
) -> Result<()> {
    normalized_token_authority.initialize(
        bump,
        receipt_token_mint.key(),
        normalized_token_mint.key(),
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
        Some(normalized_token_authority.key()),
    )?;

    Ok(())
}

pub fn process_initialize_normalized_token_pool_config(
    receipt_token_mint: &InterfaceAccount<Mint>,
    normalized_token_mint: &InterfaceAccount<Mint>,
    normalized_token_authority: &Account<NormalizedTokenAuthority>,
    normalized_token_account: &InterfaceAccount<TokenAccount>,
    normalized_token_pool_config: &mut Account<NormalizedTokenPoolConfig>,
    normalized_token_program: &Program<Token>,
    bump: u8,
) -> Result<()> {
    normalized_token_pool_config.initialize(
        bump,
        receipt_token_mint.key(),
        normalized_token_mint.key(),
        normalized_token_program.key(),
        normalized_token_authority.key(),
        normalized_token_account.key(),
    );

    Ok(())
}
