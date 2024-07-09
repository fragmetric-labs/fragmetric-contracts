pub mod constants;
pub mod error;
pub mod instructions;
pub mod fund;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use fund::*;

declare_id!("4trkJGMF6idmp7chxhEEJGqBvbYQcN4QV3iQAnYuPZet");

#[program]
pub mod restaking {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        default_protocol_fee_rate: u16,
        whitelisted_tokens: Vec<Pubkey>,
        lst_caps: Vec<u64>,
    ) -> Result<()> {
        initialize::handler(ctx, default_protocol_fee_rate, whitelisted_tokens, lst_caps)
    }
}
