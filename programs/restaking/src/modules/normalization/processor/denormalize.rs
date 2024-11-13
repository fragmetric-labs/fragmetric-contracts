use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenAccount;

use crate::modules::normalization::*;

#[inline]
pub(in crate::modules) fn denormalize_supported_token<'info>(
    normalized_token_pool_adapter: &mut NormalizedTokenPoolAdapter<'info>,
    normalized_token_account: &InterfaceAccount<'info, TokenAccount>,
    supported_token_account: &InterfaceAccount<'info, TokenAccount>,
    signer: AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    normalized_token_amount: u64,
    normalized_token_amount_as_sol: u64,
    one_supported_token_as_sol: u64,
) -> Result<()> {
    normalized_token_pool_adapter.withdraw(
        normalized_token_account,
        supported_token_account,
        signer,
        signer_seeds,
        normalized_token_amount,
        normalized_token_amount_as_sol,
        one_supported_token_as_sol
    )
}