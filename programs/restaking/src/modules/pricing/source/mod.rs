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
pub enum TokenAmount {
    SOLAmount(u64),
    TokenAmounts(Vec<(Pubkey, u64)>),
}

/// A type that can calculate the token amount as sol with its data.
pub(super) trait TokenAmountAsSOLCalculator {
    fn calculate_token_amount_as_sol(&self, amount: u64) -> Result<TokenAmount>;
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
