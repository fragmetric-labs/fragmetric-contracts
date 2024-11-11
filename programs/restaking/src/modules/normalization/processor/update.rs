use anchor_lang::prelude::*;
use anchor_spl::token_2022::spl_token_2022;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface};

use crate::modules::normalization::*;
use crate::utils::PDASeeds;

pub fn process_add_supported_token<'info>(
    supported_token_mint: &InterfaceAccount<Mint>,
    supported_token_lock_account: &InterfaceAccount<'info, TokenAccount>,
    normalized_token_pool_account: &mut Account<NormalizedTokenPoolAccount>,
    supported_token_program: &Interface<'info, TokenInterface>,
) -> Result<()> {
    normalized_token_pool_account.add_new_supported_token(
        supported_token_mint.key(),
        supported_token_program.key(),
        supported_token_lock_account.key(),
    )
}
