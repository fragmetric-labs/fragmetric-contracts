use anchor_lang::prelude::*;
use anchor_spl::token::spl_token;
use whirlpool_cpi::whirlpool::accounts::Whirlpool;

use crate::modules::pricing::{Asset, TokenValue, TokenValueProvider};

pub struct OrcaLiqPoolValueProvider;

impl TokenValueProvider for OrcaLiqPoolValueProvider {
    #[inline(never)]
    fn resolve_underlying_assets<'info>(
        self,
        token_mint: &Pubkey,
        pricing_source_accounts: &[&'info AccountInfo<'info>],
    ) -> Result<TokenValue> {
        require_eq!(pricing_source_accounts.len(), 1);

        let pool_account = Account::<Whirlpool>::try_from(pricing_source_accounts[0])?;

        require_keys_eq!(pool_account.token_mint_a, *token_mint);
        require_keys_eq!(pool_account.token_mint_b, spl_token::native_mint::ID);

        // Q64.128 into u128 numerator & 2^64 denominator
        // by multiplying 2^-64 to both numerator and denominator
        let numerator = self.calculate_price_from_sqrt(pool_account.sqrt_price);
        let mut numerator = ((numerator[2] as u128) << 64) | numerator[1] as u128;
        let mut denominator = 1u128 << 64;

        // fit to u64
        while numerator > Self::Q64_RESOLUTION {
            numerator >>= 1;
            denominator >>= 1;
        }

        Ok(TokenValue {
            numerator: vec![Asset::SOL(numerator as u64)],
            denominator: denominator.min(Self::Q64_RESOLUTION) as u64,
        })
    }
}

impl OrcaLiqPoolValueProvider {
    const Q64_RESOLUTION: u128 = u64::MAX as u128;

    /// sqrt_price is represented as Q32.64 fixed point,
    /// whose high 64bits are decimal and low 64bits are subdecimal.
    /// so, price = (hi + lo * 2^-64)^2, which can be represented as Q64.128.
    /// to prevent error, return the price as length 3 array of u64, little endian.
    fn calculate_price_from_sqrt(&self, sqrt_price: u128) -> [u64; 3] {
        let mut price = [0u64; 3];

        // 2^-64 < p < 2^64 => √p < 2^32 => hi < 2^32
        let hi = sqrt_price >> 64; // u32
        let lo = sqrt_price & Self::Q64_RESOLUTION; // u64

        let tmp = lo * lo;
        price[0] = tmp as u64;
        price[1] = (tmp >> 64) as u64;

        let tmp = 2 * hi * lo + (price[1] as u128);
        price[1] = tmp as u64;
        price[2] = (tmp >> 64) as u64;

        let tmp = hi * hi + (price[2] as u128);
        price[2] = tmp as u64;
        // 2^-64 < p < 2^64 => numerator[3] = 0
        // numerator[3] = (tmp >> 64) as u64;

        price
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_math() {
        let sqrt_price = 2 << 64;
        assert_eq!(
            OrcaLiqPoolValueProvider.calculate_price_from_sqrt(sqrt_price),
            [0, 0, 4]
        );
    }
}
