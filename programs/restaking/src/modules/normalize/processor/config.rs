use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{errors::ErrorCode, modules::normalize::*};

#[inline(always)]
pub(in crate::modules) fn get_supported_tokens_locked_amount(
    normalized_token_pool_config: &Account<NormalizedTokenPoolConfig>,
) -> Vec<(Pubkey, u64)> {
    normalized_token_pool_config.get_supported_tokens_locked_amount()
}
