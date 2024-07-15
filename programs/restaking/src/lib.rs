pub mod constants;
pub mod error;
pub mod instructions;
pub mod fund;
// pub mod oracle;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use fund::*;
// pub use oracle::*;

// mainnet: FRAGZZHbvqDwXkqaPSuKocS7EzH7rU7K6h6cW3GQAkEc
declare_id!("fragfP1Z2DXiXNuDYaaCnbGvusMP1DNQswAqTwMuY6e");

#[program]
pub mod restaking {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        receipt_token_name: String,
        default_protocol_fee_rate: u16,
        whitelisted_tokens: Vec<Pubkey>,
        lst_caps: Vec<u64>,
    ) -> Result<()> {
        instructions::initialize::handler(ctx, receipt_token_name, default_protocol_fee_rate, whitelisted_tokens, lst_caps)
    }

    pub fn deposit_sol(
        ctx: Context<DepositSOL>,
        amount: u64,
    ) -> Result<()> {
        instructions::deposit_sol::handler(ctx, amount)
    }
}
