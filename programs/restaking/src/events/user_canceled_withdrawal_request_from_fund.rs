use anchor_lang::prelude::*;

#[event]
pub struct UserCanceledWithdrawalRequestFromFund {
    pub receipt_token_mint: Pubkey,
    pub batch_id: u64,
    pub request_id: u64,

    pub user: Pubkey,
    pub user_receipt_token_account: Pubkey,
    pub user_fund_account: Pubkey,

    pub requested_supported_token_mint: Option<Pubkey>,
    pub requested_receipt_token_amount: u64,
}
