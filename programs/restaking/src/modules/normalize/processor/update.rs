use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::modules::{fund::SupportedTokenAuthority, normalize::*};

pub fn process_add_supported_token(
    supported_token_mint: &InterfaceAccount<Mint>,
    supported_token_lock_account: &InterfaceAccount<TokenAccount>,
    supported_token_account: &InterfaceAccount<TokenAccount>,
    supported_token_authority: &AccountInfo,
    normalized_token_pool_config: &mut Account<NormalizedTokenPoolConfig>,
    supported_token_program: &Interface<TokenInterface>,
) -> Result<()> {
    normalized_token_pool_config.add_supported_token(
        supported_token_mint.key(),
        supported_token_program.key(),
        supported_token_account.key(),
        supported_token_authority.key(),
        supported_token_lock_account.key(),
    )
}
