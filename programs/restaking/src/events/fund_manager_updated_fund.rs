use anchor_lang::prelude::*;

use crate::modules::fund::FundAccountInfo;

#[event]
pub struct FundManagerUpdatedFund {
    pub receipt_token_mint: Pubkey,
    pub fund_account: FundAccountInfo,
}