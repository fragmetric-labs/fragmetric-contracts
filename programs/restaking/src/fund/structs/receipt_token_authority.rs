use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ReceiptTokenAuthority {
    pub authority: Pubkey,
}
