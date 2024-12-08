use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use super::{TokenPricingSource, TokenPricingSourcePod};

#[cfg(test)]
pub use self::mock::*;

/// A type that can calculate the token amount as sol with its data.
pub trait TokenValueProvider {
    fn resolve_underlying_assets<'info>(
        self,
        token_mint: &Pubkey,
        pricing_source_accounts: &[&'info AccountInfo<'info>],
    ) -> Result<TokenValue>;
}

const TOKEN_VALUE_NUMERATOR_MAX_SIZE: usize = 20;

/// a value representing total asset value of a pricing source.
#[derive(Clone, PartialEq, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct TokenValue {
    #[max_len(TOKEN_VALUE_NUMERATOR_MAX_SIZE)]
    pub numerator: Vec<Asset>,
    pub denominator: u64,
}

impl TokenValue {
    /// indicates whether the token is not a kind of basket such as normalized token,
    /// so the value of the token can be resolved by one self without other token information.
    pub fn is_atomic(&self) -> bool {
        self.numerator.iter().all(|asset| match asset {
            Asset::Token(..) => false,
            Asset::SOL(..) => true,
        })
    }
}

impl std::fmt::Display for TokenValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let numerator = self
            .numerator
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        f.debug_struct("TokenValue")
            .field("atomic", &self.is_atomic())
            .field("numerator", &numerator)
            .field("denominator", &self.denominator)
            .finish()
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod, Debug)]
#[repr(C, align(8))]
pub struct TokenValuePod {
    pub numerator: [AssetPod; TOKEN_VALUE_NUMERATOR_MAX_SIZE],
    num_numerator: u64,
    pub denominator: u64,
}

impl From<TokenValue> for TokenValuePod {
    fn from(src: TokenValue) -> Self {
        let mut pod = TokenValuePod::zeroed();
        pod.num_numerator = src.numerator.len() as u64;
        for (i, asset) in src.numerator.into_iter().take(TOKEN_VALUE_NUMERATOR_MAX_SIZE).enumerate() {
            pod.numerator[i] = asset.into();
        }
        pod.denominator = src.denominator;
        pod
    }
}

impl From<&TokenValuePod> for TokenValue {
    fn from(pod: &TokenValuePod) -> Self {
        Self {
            numerator: pod
                .numerator
                .iter()
                .filter_map(Into::into)
                .collect::<Vec<_>>(),
            denominator: pod.denominator,
        }
    }
}

#[derive(Clone, PartialEq, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum Asset {
    // amount
    SOL(u64),
    // mint, known pricing source, amount
    Token(Pubkey, Option<TokenPricingSource>, u64),
}

impl std::fmt::Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SOL(amount) => write!(f, "{} SOL", amount),
            Self::Token(mint, Some(source), amount) => {
                write!(f, "{} TOKEN({}, source={:?})", amount, mint, source)
            }
            Self::Token(mint, None, amount) => write!(f, "{} TOKEN({})", amount, mint),
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod, Debug)]
#[repr(C)]
pub struct AssetPod {
    discriminant: u8,
    _padding: [u8; 7],
    sol_amount: u64,
    token_amount: u64,
    token_mint: Pubkey,
    token_pricing_source: TokenPricingSourcePod,
}

impl From<Asset> for AssetPod {
    fn from(src: Asset) -> Self {
        match src {
            Asset::SOL(sol_amount) => Self {
                discriminant: 1,
                _padding: [0; 7],
                sol_amount,
                token_amount: 0,
                token_mint: Pubkey::default(),
                token_pricing_source: TokenPricingSourcePod::default(),
            },
            Asset::Token(token_mint, token_pricing_source, token_amount) => Self {
                discriminant: 2,
                _padding: [0; 7],
                sol_amount: 0,
                token_amount,
                token_mint,
                token_pricing_source: token_pricing_source.map(Into::into).unwrap_or_default(),
            },
        }
    }
}

impl From<&AssetPod> for Option<Asset> {
    fn from(pod: &AssetPod) -> Self {
        match pod.discriminant {
            0 => None,
            1 => Some(Asset::SOL(pod.sol_amount)),
            2 => Some(Asset::Token(
                pod.token_mint,
                (&pod.token_pricing_source).into(),
                pod.token_amount,
            )),
            _ => panic!("invalid discriminant for TokenPricingSource"),
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
                        MockAsset::Token(mint, amount) => Asset::Token(mint, None, amount),
                    })
                    .collect(),
                denominator: *self.denominator,
            })
        }
    }
}
