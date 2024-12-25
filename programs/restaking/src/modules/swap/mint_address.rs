use anchor_lang::prelude::*;
use anchor_spl::token::spl_token;

/// Trait for the types that represents token mint address.
pub trait MintAddress {
    fn mint_address() -> Pubkey;
    fn is_native_mint() -> bool {
        Self::mint_address() == spl_token::native_mint::ID
    }
}

pub struct NativeMint;

impl MintAddress for NativeMint {
    fn mint_address() -> Pubkey {
        spl_token::native_mint::ID
    }
}

// add more tokens here...
