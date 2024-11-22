use anchor_lang::prelude::*;

#[cfg(test)]
pub use self::mock::*;

use super::TokenPricingSource;

/// A type that can calculate the token amount as sol with its data.
pub trait TokenValueProvider {
    fn resolve_underlying_assets<'info>(
        self,
        token_mint: &Pubkey,
        pricing_source_accounts: &[&'info AccountInfo<'info>],
    ) -> Result<TokenValue>;
}

/// a value representing total asset value of a pricing source.
#[derive(Clone, PartialEq, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct TokenValue {
    #[max_len(20)]
    pub numerator: Vec<Asset>,
    pub denominator: u64,
}

impl std::fmt::Display for TokenValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let numerator = self
            .numerator
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        f.debug_struct("TokenValue")
            .field("numerator", &numerator)
            .field("denominator", &self.denominator)
            .finish()
    }
}

#[derive(Clone, PartialEq, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum Asset {
    // amount
    SOL(u64),
    // mint, known pricing source, amount
    TOKEN(Pubkey, Option<TokenPricingSource>, u64),
}

impl std::fmt::Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SOL(amount) => write!(f, "{} SOL", amount),
            Self::TOKEN(mint, Some(source), amount) => {
                write!(f, "{} TOKEN({}, source={:?})", amount, mint, source)
            }
            Self::TOKEN(mint, None, amount) => write!(f, "{} TOKEN({})", amount, mint),
        }
    }
}

#[cfg(test)]
mod mock {
    use super::*;

    #[derive(Clone, Debug, InitSpace, AnchorSerialize, AnchorDeserialize, Copy, PartialEq, Eq)]
    pub enum MockAsset {
        // amount
        SOL(u64),
        // mint, amount
        Token(Pubkey, u64),
    }

    impl std::fmt::Display for MockAsset {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::SOL(amount) => write!(f, "{} SOL", amount),
                Self::Token(mint, amount) => write!(f, "{} TOKEN({})", amount, mint),
            }
        }
    }

    /// Example Mock Provider; Price: 1 denominated unit = 1.2 lamports
    pub struct MockPricingSourceValueProvider<'a> {
        numerator: &'a Vec<MockAsset>,
        denominator: &'a u64,
    }

    impl<'a> MockPricingSourceValueProvider<'a> {
        pub fn new(numerator: &'a Vec<MockAsset>, denominator: &'a u64) -> Self {
            Self {
                numerator,
                denominator,
            }
        }
    }

    impl<'b> TokenValueProvider for MockPricingSourceValueProvider<'b> {
        fn resolve_underlying_assets<'info>(
            self,
            _token_mint: &Pubkey,
            pricing_source_accounts: &[&'info AccountInfo<'info>],
        ) -> Result<TokenValue> {
            require_eq!(pricing_source_accounts.len(), 0);

            Ok(TokenValue {
                numerator: self
                    .numerator
                    .iter()
                    .map(|&asset| match asset {
                        MockAsset::SOL(amount) => Asset::SOL(amount),
                        MockAsset::Token(mint, amount) => Asset::TOKEN(mint, None, amount),
                    })
                    .collect(),
                denominator: *self.denominator,
            })
        }
    }
}
