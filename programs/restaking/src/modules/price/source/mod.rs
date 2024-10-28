use anchor_lang::prelude::*;

mod marinade_stake_pool;
#[cfg(test)]
mod mock;
mod spl_stake_pool;

pub(super) use marinade_stake_pool::*;
#[cfg(test)]
pub(super) use mock::*;
pub(super) use spl_stake_pool::*;

#[derive(Clone, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum TokenValue {
    SOL(u64),
    Tokens(Vec<(Pubkey, u64)>),
}

/// A type that can calculate the value of token with its data.
pub(super) trait TokenValueCalculator {
    fn calculate_token_value(&self, amount: u64) -> Result<TokenValue>;
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
#[non_exhaustive]
pub enum TokenPricingSource {
    SPLStakePool {
        address: Pubkey,
    },
    MarinadeStakePool {
        address: Pubkey,
    },
    #[cfg(test)]
    Mock,
}
