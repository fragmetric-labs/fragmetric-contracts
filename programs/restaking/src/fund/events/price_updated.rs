use anchor_lang::prelude::*;

use crate::fund::*;

#[event]
pub struct FundPriceUpdated {
    pub lrt_mint: Pubkey,
    pub lrt_price: u64,
    pub fund_info: FundInfo,
}
