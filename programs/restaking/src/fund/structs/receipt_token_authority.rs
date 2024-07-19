use anchor_lang::prelude::*;

#[account]
pub struct ReceiptTokenAuthority {
    pub authority: Pubkey,
}
