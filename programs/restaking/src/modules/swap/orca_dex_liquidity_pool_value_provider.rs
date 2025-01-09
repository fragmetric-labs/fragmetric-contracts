use anchor_lang::prelude::*;
use anchor_spl::token::spl_token;
use whirlpool_cpi::whirlpool::accounts::Whirlpool;

use crate::modules::pricing::{Asset, TokenValue, TokenValueProvider};

pub struct OrcaDEXLiquidityPoolValueProvider;

impl TokenValueProvider for OrcaDEXLiquidityPoolValueProvider {
    #[inline(never)]
    fn resolve_underlying_assets<'info>(
        self,
        token_value_to_update: &mut TokenValue,
        token_mint: &Pubkey,
        pricing_source_accounts: &[&'info AccountInfo<'info>],
    ) -> Result<()> {
        require_eq!(pricing_source_accounts.len(), 1);

        let whirlpool = Account::<Whirlpool>::try_from(pricing_source_accounts[0])?;

        let (is_inverse, base_token_mint) = if whirlpool.token_mint_a == *token_mint {
            (false, whirlpool.token_mint_b)
        } else {
            require_keys_eq!(whirlpool.token_mint_b, *token_mint);
            (true, whirlpool.token_mint_a)
        };

        // First, calculate price from pool account.
        //
        // The calculated price is notation Q64.128,
        // which means, there are 128 bits behind decimal point.
        //
        // Since Q64.128 notation requires 192 bits,
        // A return value is an length 3 array of 64-bit integer.
        //
        // Note that array indexing follows little endianness,
        // so `(price[2] << 128) + (price[1] << 64) + price[0]` is the actual
        // Q64.128 notation.
        let price = self.calculate_price_from_sqrt(whirlpool.sqrt_price);

        // fit both numerator and denominator into 64-bit integer
        // by reducing the number of  significant digits.
        let (numerator, denominator) = self.fit_price_into_u64(price, is_inverse);

        // Check base mint
        let asset = match base_token_mint {
            spl_token::native_mint::ID => Asset::SOL(numerator),
            mint => Asset::Token(mint, None, numerator),
        };

        token_value_to_update.numerator.clear();
        token_value_to_update.numerator.reserve_exact(1);

        token_value_to_update.numerator.extend([asset]);
        token_value_to_update.denominator = denominator;

        Ok(())
    }
}

impl OrcaDEXLiquidityPoolValueProvider {
    /// In orca pool, sqrt_price is a square root value of the price,
    /// which is represented as Q32.64 fixed point decimal notation.
    ///
    /// Qm.n fixed point decimal notation uses (m+n) bit integer,
    /// implying that there are n bits behind decimal point.
    /// In other words, high m bits of (m+n) bits are integer parts
    /// while low n bits are fractional parts.
    /// For example, Q4.4 notation of 3.75(0b11.11) is `0b0011_1100`.
    ///
    /// The power of 2 of Q32.64 value can be represented as Q64.128 notation,
    /// which requires 192 bits. This function splits 192 bits into three
    /// 64-bit integer and stores in a fixed size array of length 3.
    ///
    /// Note that array indexing follows little endianness,
    /// so `(price[2] << 128) + (price[1] << 64) + price[0]` is the actual
    /// Q64.128 notation.
    fn calculate_price_from_sqrt(&self, sqrt_price: u128) -> [u64; 3] {
        // here we perform simple binary multiplication with chunk size = 64 bit.
        //                     hi       lo
        //      X              hi       lo
        // -------------------------------
        //                  hi*lo    lo*lo
        //         hi*hi    lo*hi
        // -------------------------------
        //         hi*hi  2*hi*lo    lo*lo

        // First we split sqrt_price into high 32 bits and low 64 bits.
        let hi = sqrt_price >> 64;
        let lo = sqrt_price & 0xFFFF_FFFF_FFFF_FFFF;

        let mut price = [0u64; 3];
        let mut carry = 0u128;

        // Start simple binary multiplication.
        let tmp = lo * lo + carry;
        price[0] = tmp as u64;
        carry = tmp >> 64;

        let tmp = 2 * hi * lo + carry;
        price[1] = tmp as u64;
        carry = tmp >> 64;

        let tmp = hi * hi + carry;
        price[2] = tmp as u64;

        // Final carry must be zero.
        #[cfg(test)]
        {
            carry = tmp >> 64;
            assert_eq!(carry, 0);
        }

        price
    }

    /// To convert Q64.128 price into `TokenValue`,
    /// we need to approximate the price by reducing
    /// the number of significant bits of scientific notation.
    ///
    /// First, convert price into fraction.
    /// Let's denote numerator as N and denominator as M.
    ///
    /// ```txt
    ///                         N      (price[2] << 128) + (price[1] << 64) + price[0]
    ///    price_as_fraction = --- = ---------------------------------------------------
    ///                         M                           2^128
    /// ```
    ///
    /// To reduce the number of significant bits, we can shift both N and M to right.
    /// M is 129 bits, so we need to shift at least 65 times.
    ///
    /// Hopefully, since 2^-64 < p < 2^64 is guaranteed,
    /// we know that 2^64 < N = p * 2^128 < 2^192, so N is at least 65 bits and at most 192 bits.
    /// Therefore we can shift at least 64 times.
    ///
    /// ```txt
    ///                         N      (price[2] << 64) + price[1]
    ///    price_after_shift = --- = -------------------------------
    ///                         M                  2^64
    /// ```
    ///
    /// Now, only one more shift will make M to fit to 64-bit integer.
    /// If N needs more shift, we don't have to care about M anymore.
    /// Let's fit N to 64-bit integer.
    ///
    /// Otherwise, if N already fits to 64-bit integer,
    /// instead of shift, we can replace M(= 2^64) into 2^64-1,
    /// allowing very small, ignorable error.
    fn fit_price_into_u64(&self, price: [u64; 3], is_inverse: bool) -> (u64, u64) {
        let mut n = ((price[2] as u128) << 64) | price[1] as u128;
        let mut m = 1u128 << 64;

        // fit N to u64
        while n > 0xFFFF_FFFF_FFFF_FFFF {
            n >>= 1;
            m >>= 1;
        }

        // if there were no shift at while loop
        if m > 0xFFFF_FFFF_FFFF_FFFF {
            #[cfg(test)]
            {
                assert_eq!(m, 1u128 << 64);
            }

            m = 0xFFFF_FFFF_FFFF_FFFF;
        }

        if is_inverse {
            (m as u64, n as u64)
        } else {
            (n as u64, m as u64)
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::utils::tests::MockAccountsDb;

    use super::*;

    // Min/Max sqrt_price to test
    const MIN_SQRT_PRICE_X64: u128 = 1 << 48; // √p = 2^-16
    const MAX_SQRT_PRICE_X64: u128 = 1 << 80; // √p = 2^16

    proptest! {
        #[test]
        fn test_resolve_token_pricing_source(
            sqrt_price in MIN_SQRT_PRICE_X64..MAX_SQRT_PRICE_X64,
        ) {
            run_test_with_params(sqrt_price);
        }
    }

    fn run_test_with_params(sqrt_price: u128) {
        let buffer = [0u8; 653];
        let mut whirlpool = Whirlpool::try_deserialize_unchecked(&mut buffer.as_slice()).unwrap();
        whirlpool.sqrt_price = sqrt_price;
        let token_amount_a = 1_000_000_000;

        let pool_address = Pubkey::new_unique();
        let mut data = vec![];

        whirlpool.token_mint_a = Pubkey::new_unique();
        whirlpool.token_mint_b = spl_token::native_mint::ID;
        whirlpool
            .try_serialize(&mut data)
            .expect("Failed to serialize data");

        let mut accounts = MockAccountsDb::default();
        accounts.add_or_update_accounts(pool_address, 0, data, whirlpool_cpi::whirlpool::ID, false);
        accounts.run_with_accounts(
            &[AccountMeta::new_readonly(pool_address, false)],
            move |pricing_source_accounts| {
                let mut token_value = TokenValue::default();
                OrcaDEXLiquidityPoolValueProvider
                    .resolve_underlying_assets(
                        &mut token_value,
                        &whirlpool.token_mint_a,
                        &[&pricing_source_accounts[0]],
                    )
                    .expect("Failed to resolve underlying asset");

                let numerator = token_value.numerator;
                assert_eq!(numerator.len(), 1);
                let Asset::SOL(numerator) = numerator[0] else {
                    panic!("Asset must be SOL");
                };

                let sqrt_price_f64 = whirlpool.sqrt_price as f64 / (1u128 << 64) as f64;
                let price_f64 = sqrt_price_f64 * sqrt_price_f64;

                let denominator = token_value.denominator;
                let resolved_token_amount_a_as_b =
                    crate::utils::get_proportional_amount(token_amount_a, numerator, denominator)
                        .expect("Arithmetic error");
                let expected_token_amount_a_as_b = (token_amount_a as f64 * price_f64) as u64;
                let diff = resolved_token_amount_a_as_b.abs_diff(expected_token_amount_a_as_b);
                let error = if expected_token_amount_a_as_b == 0 {
                    if diff == 0 {
                        0f32
                    } else {
                        1f32
                    }
                } else {
                    diff as f32 / expected_token_amount_a_as_b as f32
                };

                // Acceptance Criteria: price error within 2^-14 ~= 0.006%
                assert!(error < 1e-14);
            },
        );
    }
}
