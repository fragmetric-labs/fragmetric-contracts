use anchor_lang::prelude::*;

#[event]
pub struct OperatorDonatedToFund {
    pub receipt_token_mint: Pubkey,
    pub fund_account: Pubkey,
    pub supported_token_mint: Option<Pubkey>,
    pub donated_amount: u64,
    pub deposited_amount: u64,
    pub offsetted_receivable_amount: u64,
}
