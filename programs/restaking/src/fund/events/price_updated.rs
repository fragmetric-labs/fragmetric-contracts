use anchor_lang::prelude::*;

use crate::fund::*;

#[event]
pub struct FundPriceUpdated {
    pub receipt_token_mint: Pubkey,
    pub receipt_token_price: u64,
    pub fund_info: FundInfo,
}
