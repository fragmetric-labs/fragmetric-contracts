use anchor_lang::prelude::*;

#[event]
pub struct FundWithdrawalRequestCanceled {
    pub user: Pubkey,
    pub user_lrt_account: Pubkey,
    pub user_receipt_account: Pubkey, // Receipt of withdrawal request

    pub request_id: u64,
    pub lrt_mint: Pubkey,
    pub lrt_amount: u64,
}
