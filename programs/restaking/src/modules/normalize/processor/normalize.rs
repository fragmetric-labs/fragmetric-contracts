use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenAccount;

use crate::modules::normalize::*;

pub(in crate::modules) fn normalize_supported_token<'info>(
    normalized_token_pool_adapter: &mut NormalizedTokenPoolAdapter<'info>,
    normalized_token_account: &InterfaceAccount<'info, TokenAccount>,
    supported_token_account: &InterfaceAccount<'info, TokenAccount>,
    signer: AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    supported_token_amount: u64,
    supported_token_amount_as_sol: u64,
    one_normalized_token_as_sol: u64,
) -> Result<()> {
    normalized_token_pool_adapter.deposit(
        normalized_token_account,
        supported_token_account,
        signer,
        signer_seeds,
        supported_token_amount,
        supported_token_amount_as_sol,
        one_normalized_token_as_sol,
    )
}
