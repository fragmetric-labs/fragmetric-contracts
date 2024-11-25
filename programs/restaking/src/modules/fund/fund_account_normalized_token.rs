use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct NormalizedToken {
    pub(super) mint: Pubkey,
    pub(super) program: Pubkey,
    pub(super) decimals: u8,
    pub(super) pool: Pubkey,
    pub(super) one_token_as_sol: u64,
    pub(super) operation_reserved_amount: u64,
    _reserved: [u8; 64],
}

impl NormalizedToken {
    pub(super) fn new(mint: Pubkey, program: Pubkey, decimals: u8, pool: Pubkey) -> Self {
        Self {
            mint,
            program,
            decimals,
            pool,
            one_token_as_sol: 0,
            operation_reserved_amount: 0,
            _reserved: [0; 64],
        }
    }
}
