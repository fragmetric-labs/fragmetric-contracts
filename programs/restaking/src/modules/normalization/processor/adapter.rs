// use anchor_lang::prelude::*;
//
// use crate::modules::normalization::*;
//
// #[inline(always)]
// pub(in crate::modules) fn create_normalized_token_pool_adapter<'info>(
//     normalized_token_pool_account: &Account<'info, NormalizedTokenPoolAccount>,
//     adapter_constructor_accounts: &[&'info AccountInfo<'info>],
// ) -> Result<NormalizedTokenPoolAdapter<'info>> {
//     NormalizedTokenPoolAdapter::new(
//         normalized_token_pool_account.clone(),
//         adapter_constructor_accounts,
//     )
// }
