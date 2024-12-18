use anchor_lang::prelude::*;

#[event]
pub struct OperatorUpdatedFundPrices {
    pub receipt_token_mint: Pubkey,
    pub fund_account: Pubkey,
}
