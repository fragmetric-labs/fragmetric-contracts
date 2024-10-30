use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::errors::ErrorCode;
use crate::modules::normalize::*;

pub(in crate::modules) fn normalize_supported_token<'info>(
    normalized_token_pool_adapter: &mut NormalizedTokenPoolAdapter<'info>,
    supported_token_authority: AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    supported_token_amount: u64,
    supported_token_amount_as_sol: u64,
    one_normalized_token_as_sol: u64,
) -> Result<()> {
    let normalized_token_mint_amount = crate::utils::get_proportional_amount(
        supported_token_amount_as_sol,
        normalized_token_pool_adapter.get_denominated_amount_per_normalized_token()?,
        one_normalized_token_as_sol,
    )
    .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

    normalized_token_pool_adapter.deposit(
        supported_token_authority,
        signer_seeds,
        supported_token_amount,
        normalized_token_mint_amount,
    )
}
