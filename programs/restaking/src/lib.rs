pub mod constants;
pub mod error;
pub mod fund;
pub mod instructions;
// pub mod oracle;

use anchor_lang::prelude::*;

pub use constants::*;
pub use error::*;
pub use fund::*;
pub use instructions::*;
// pub use oracle::*;

#[cfg(feature = "mainnet")]
declare_id!("FRAGZZHbvqDwXkqaPSuKocS7EzH7rU7K6h6cW3GQAkEc");
#[cfg(not(feature = "mainnet"))]
// declare_id!("fragfP1Z2DXiXNuDYaaCnbGvusMP1DNQswAqTwMuY6e");
declare_id!("9UpfJBgVKuZ1EzowJL6qgkYVwv3HhLpo93aP8L1QW86D");

#[program]
pub mod restaking {
    use super::*;

    pub fn fund_initialize(
        ctx: Context<InitializeFund>,
        receipt_token_name: String,
        default_protocol_fee_rate: u16,
        tokens: Vec<TokenInfo>
    ) -> Result<()> {
        InitializeFund::handler(
            ctx,
            receipt_token_name,
            default_protocol_fee_rate,
            tokens,
        )
    }

    pub fn fund_add_whitelisted_token(
        ctx: Context<FundUpdateToken>,
        token: Pubkey,
        token_cap: u64
    ) -> Result<()> {
        FundUpdateToken::add_whitelisted_token(ctx, token, token_cap)
    }

    pub fn fund_update_token_info(
        ctx: Context<FundUpdateToken>,
        token: Pubkey,
        info: TokenInfo
    ) -> Result<()> {
        FundUpdateToken::update_token_info(ctx, token, info)
    }

    pub fn fund_update_default_protocol_fee_rate(
        ctx: Context<FundUpdateToken>,
        default_protocol_fee_rate: u16
    ) -> Result<()> {
        FundUpdateToken::update_default_protocol_fee_rate(ctx, default_protocol_fee_rate)
    }

    pub fn deposit_sol(ctx: Context<DepositSOL>, amount: u64) -> Result<()> {
        DepositSOL::handler(ctx, amount)
    }
}
