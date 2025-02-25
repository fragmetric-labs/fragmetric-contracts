use anchor_lang::prelude::*;
use bytemuck::Zeroable;

use crate::modules::swap::{TokenSwapSource, TokenSwapSourcePod};

#[zero_copy]
#[repr(C)]
pub(super) struct TokenSwapStrategy {
    pub mints: [Pubkey; 2],
    pub swap_source: TokenSwapSourcePod,
    _reserved: [u8; 128],
}

impl TokenSwapStrategy {
    pub fn initialize(&mut self, mints: [Pubkey; 2], swap_source: TokenSwapSource) {
        *self = Zeroable::zeroed();

        self.mints = mints;
        swap_source.serialize_as_pod(&mut self.swap_source);
    }

    pub fn is_swap_pair(&self, from_token_mint: Pubkey, to_token_mint: Pubkey) -> bool {
        self.mints == [from_token_mint, to_token_mint]
            || self.mints == [to_token_mint, from_token_mint]
    }
}
