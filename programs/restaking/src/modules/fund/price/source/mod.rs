use anchor_lang::prelude::*;

mod marinade_stake_pool;
mod spl_stake_pool;

pub use marinade_stake_pool::*;
pub use spl_stake_pool::*;

/// A type that can calculate the token price with its data.
pub trait TokenPriceCalculator {
    fn calculate_token_price(&self, token_amount: u64) -> Result<u64>;
}

/// Try to deserialize an account from [AccountInfo],
/// when a single account is a pricing source.
pub trait ToCalculator<T: TokenPriceCalculator + AccountDeserialize + Owner> {
    fn to_calculator_checked(&self) -> Result<T>;
}

impl<'info, T> ToCalculator<T> for AccountInfo<'info>
where
    T: TokenPriceCalculator + AccountDeserialize + Owner,
{
    fn to_calculator_checked(&self) -> Result<T> {
        if self.owner == &anchor_lang::solana_program::system_program::ID && self.lamports() == 0 {
            return Err(ErrorCode::AccountNotInitialized.into());
        }

        if self.owner != &T::owner() {
            return Err(Error::from(ErrorCode::AccountOwnedByWrongProgram)
                .with_pubkeys((*self.owner, T::owner())));
        }

        let mut data: &[u8] = &self.try_borrow_data()?;
        T::try_deserialize(&mut data)
    }
}
