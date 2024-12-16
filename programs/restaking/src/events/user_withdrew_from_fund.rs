use anchor_lang::prelude::*;

#[event]
pub struct UserWithdrewFromFund {
    pub receipt_token_mint: Pubkey,
    pub fund_account: Pubkey,
    pub supported_token_mint: Option<Pubkey>,

    pub user: Pubkey,
    pub user_receipt_token_account: Pubkey,
    pub user_fund_account: Pubkey,
    pub user_supported_token_account: Option<Pubkey>,

    pub fund_withdrawal_batch_account: Pubkey,
    pub batch_id: u64,
    pub request_id: u64,
    pub burnt_receipt_token_amount: u64,
    pub returned_receipt_token_amount: u64,
    pub withdrawn_amount: u64,
    pub deducted_fee_amount: u64,
}
