use anchor_lang::prelude::*;
use bytemuck::Zeroable;

use crate::errors::ErrorCode;
use crate::modules::swap::{TokenSwapSource, TokenSwapSourcePod};

/// A strategy to swap `from_token` to `to_token`.
///
/// There are two restrictions that will be loosen in the future.
/// These restrictions are not checked on-chain, so fund manager
/// should carefully configure.
///
/// 1. Each token can be swapped to at most one type of token.
/// In other words, there cannot exist two strategies with same `from_token`.
/// In the future, when there are two or more strategies with same `from_token`,
/// then swap amount will be distributed among those strategies.
///
/// 2. Each token must be swapped to one of fund's supported tokens.
/// In the future, operation cycle will support multiple-hop swap, for example, A -> B -> C.
/// In this case `to_token` need not be one of fund's supported token.
/// However, to prevent endless swap, token swap strategies must form DAG(vertex = token, edge = strategy).
#[zero_copy]
#[repr(C)]
pub(super) struct TokenSwapStrategy {
    pub from_token_mint: Pubkey,
    pub to_token_mint: Pubkey,
    pub swap_source: TokenSwapSourcePod,
    _reserved: [u8; 128],
}

impl TokenSwapStrategy {
    pub fn initialize(
        &mut self,
        from_token_mint: Pubkey,
        to_token_mint: Pubkey,
        swap_source: TokenSwapSource,
    ) -> Result<()> {
        match swap_source {
            TokenSwapSource::OrcaDEXLiquidityPool { .. } => {}
        }

        *self = Zeroable::zeroed();

        self.from_token_mint = from_token_mint;
        self.to_token_mint = to_token_mint;
        swap_source.serialize_as_pod(&mut self.swap_source);

        Ok(())
    }
}
