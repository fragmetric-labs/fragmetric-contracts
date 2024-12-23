use super::{TokenPricingSource, TokenPricingSourcePod, PRICING_SERVICE_EXPECTED_TOKENS_SIZE};
use crate::{errors, utils};
use anchor_lang::prelude::*;

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

const TOKEN_VALUE_MAX_NUMERATORS_SIZE: usize = PRICING_SERVICE_EXPECTED_TOKENS_SIZE + 1;

/// a value representing total asset value of a pricing source.
#[derive(Clone, PartialEq, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct TokenValue {
    #[max_len(TOKEN_VALUE_MAX_NUMERATORS_SIZE)]
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
            .field("atomic", &self.is_atomic())
            .field("numerator", &numerator)
            .field("denominator", &self.denominator)
            .finish()
    }
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

    pub fn add(&mut self, asset: Asset) {
        match &asset {
            Asset::SOL(sol_amount) => {
                for asset in &mut self.numerator {
                    match asset {
                        Asset::SOL(existing_sol_amount) => {
                            *existing_sol_amount += *sol_amount;
                            return;
                        }
                        _ => (),
                    }
                }
                self.numerator.push(asset);
            }
            Asset::Token(token_mint, token_pricing_source, token_amount) => {
                for asset in &mut self.numerator {
                    match asset {
                        Asset::Token(
                            existing_token_mint,
                            existing_token_pricing_source,
                            existing_token_amount,
                        ) => {
                            if existing_token_mint == token_mint {
                                *existing_token_amount += *token_amount;
                                if existing_token_pricing_source.is_none()
                                    && token_pricing_source.is_some()
                                {
                                    *existing_token_pricing_source = token_pricing_source.clone();
                                }
                                return;
                            }
                        }
                        _ => (),
                    }
                }
                self.numerator.push(asset);
            }
        }
    }

    pub fn serialize_as_pod(&self, pod: &mut TokenValuePod) -> Result<()> {
        if self.numerator.len() > TOKEN_VALUE_MAX_NUMERATORS_SIZE {
            err!(errors::ErrorCode::IndexOutOfBoundsException)?;
        }
        pod.num_numerator = self.numerator.len() as u64;
        for (i, asset) in self.numerator.iter().enumerate() {
            asset.serialize_as_pod(&mut pod.numerator[i]);
        }
        pod.denominator = self.denominator;

        Ok(())
    }
}

#[zero_copy]
#[derive(Debug)]
#[repr(C)]
pub struct TokenValuePod {
    pub numerator: [AssetPod; TOKEN_VALUE_MAX_NUMERATORS_SIZE],
    pub num_numerator: u64,
    pub denominator: u64,
}

impl TokenValuePod {
    pub fn try_deserialize(&self) -> Result<TokenValue> {
        Ok(TokenValue {
            numerator: self
                .numerator
                .iter()
                .take(self.num_numerator as usize)
                .map(|pod| pod.try_deserialize())
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .filter_map(|a| a)
                .collect(),
            denominator: self.denominator,
        })
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

impl Asset {
    fn serialize_as_pod(&self, pod: &mut AssetPod) {
        match self {
            Asset::SOL(sol_amount) => {
                pod.discriminant = 1;
                pod.sol_amount = *sol_amount;
                pod.token_amount = 0;
                pod.token_mint = Pubkey::default();
                pod.token_pricing_source.clear();
            }
            Asset::Token(token_mint, token_pricing_source, token_amount) => {
                pod.discriminant = 2;
                pod.sol_amount = 0;
                pod.token_amount = *token_amount;
                pod.token_mint = *token_mint;
                match token_pricing_source {
                    Some(source) => {
                        source.serialize_as_pod(&mut pod.token_pricing_source);
                    }
                    None => {
                        pod.token_pricing_source.clear();
                    }
                }
            }
        }
    }
}

#[zero_copy]
#[derive(Debug)]
#[repr(C)]
pub struct AssetPod {
    pub discriminant: u8,
    _padding: [u8; 7],
    pub sol_amount: u64,
    pub token_amount: u64,
    pub token_mint: Pubkey,
    pub token_pricing_source: TokenPricingSourcePod,
}

impl AssetPod {
    fn try_deserialize(&self) -> Result<Option<Asset>> {
        Ok(if self.discriminant == 0 {
            None
        } else {
            Some(match self.discriminant {
                1 => Asset::SOL(self.sol_amount),
                2 => Asset::Token(
                    self.token_mint,
                    self.token_pricing_source.try_deserialize()?,
                    self.token_amount,
                ),
                _ => Err(Error::from(ProgramError::InvalidAccountData))?,
            })
        })
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
