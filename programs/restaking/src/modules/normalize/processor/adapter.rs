use anchor_lang::prelude::*;

use crate::modules::normalize::*;

#[inline(always)]
pub(in crate::modules) fn create_normalized_token_pool_adapter<'info>(
    normalized_token_pool_config: &Account<'info, NormalizedTokenPoolConfig>,
    adapter_constructor_accounts: &'info [AccountInfo<'info>],
) -> Result<NormalizedTokenPoolAdapter<'info>> {
    NormalizedTokenPoolAdapter::new(
        normalized_token_pool_config.clone(),
        adapter_constructor_accounts,
    )
}
