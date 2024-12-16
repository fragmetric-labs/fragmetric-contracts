use anchor_lang::prelude::*;

#[event]
pub struct UserTransferredReceiptToken {
    pub receipt_token_mint: Pubkey,
    pub fund_account: Pubkey,
    pub updated_user_reward_accounts: Vec<Pubkey>,

    pub source: Pubkey, // user(token account owner)
    pub source_receipt_token_account: Pubkey,
    pub source_fund_account: Option<Pubkey>,

    pub destination: Pubkey, // user(token account owner)
    pub destination_receipt_token_account: Pubkey,
    pub destination_fund_account: Option<Pubkey>,

    pub transferred_receipt_token_amount: u64,
}
