use anchor_lang::prelude::*;
use crate::modules::fund::FundInfo;

#[event]
pub struct UserUpdatedFundPrice {
    pub receipt_token_mint: Pubkey,
    pub fund_info: FundInfo,
}
