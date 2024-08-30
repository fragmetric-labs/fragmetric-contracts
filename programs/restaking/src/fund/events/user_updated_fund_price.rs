use anchor_lang::prelude::*;

use crate::fund::*;

#[event]
pub struct UserUpdatedFundPrice {
    pub receipt_token_mint: Pubkey,
    pub fund_info: FundInfo,
}
