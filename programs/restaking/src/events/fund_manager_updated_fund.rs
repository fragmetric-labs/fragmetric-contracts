use anchor_lang::prelude::*;

#[event]
pub struct FundManagerUpdatedFund {
    pub receipt_token_mint: Pubkey,
    pub fund_account: Pubkey,
}
