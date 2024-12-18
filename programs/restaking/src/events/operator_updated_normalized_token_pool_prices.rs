use anchor_lang::prelude::*;

#[event]
pub struct OperatorUpdatedNormalizedTokenPoolPrices {
    pub normalized_token_mint: Pubkey,
    pub normalized_token_pool_account: Pubkey,
}
