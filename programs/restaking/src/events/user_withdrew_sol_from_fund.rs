use anchor_lang::prelude::*;

#[event]
pub struct UserWithdrewSOLFromFund {
    pub receipt_token_mint: Pubkey,
    pub batch_id: u64,
    pub request_id: u64,

    pub user: Pubkey,
    pub user_receipt_token_account: Pubkey,
    pub user_fund_account: Pubkey,

    pub burnt_receipt_token_amount: u64,
    pub withdrawn_sol_amount: u64,
    pub deducted_sol_fee_amount: u64,

    pub fund_withdrawal_batch_account: Pubkey,
    pub fund_account: Pubkey,
}
