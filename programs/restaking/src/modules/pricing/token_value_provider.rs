use anchor_lang::prelude::*;

use crate::errors::ErrorCode;

use super::{TokenPricingSource, TokenPricingSourcePod};

#[cfg(test)]
pub use self::mock::*;

/// A type that can calculate the token amount as sol with its data.
pub trait TokenValueProvider {
    fn resolve_underlying_assets<'info>(
        self,
        token_mint: &Pubkey,
        pricing_source_accounts: &[&'info AccountInfo<'info>],
        result: &mut TokenValue,
    ) -> Result<()>;
}

const TOKEN_VALUE_MAX_NUMERATORS_SIZE: usize = 33;

/// a value representing total asset value of a pricing source.
#[derive(Clone, PartialEq, InitSpace, AnchorSerialize, AnchorDeserialize, Default)]
#[cfg_attr(test, derive(Debug))]
pub struct TokenValue {
    #[max_len(TOKEN_VALUE_MAX_NUMERATORS_SIZE)]
    pub numerator: Vec<Asset>,
    pub denominator: u64,
}

impl TokenValue {
    pub const MAX_NUMERATOR_SIZE: usize = TOKEN_VALUE_MAX_NUMERATORS_SIZE;

    /// indicates whether the token is not a kind of basket such as normalized token,
    /// so the value of the token can be resolved by one self without other token information.
    pub fn is_atomic(&self) -> bool {
        self.numerator
            .iter()
            .all(|asset| matches!(asset, Asset::SOL(_)))
    }

    pub(super) fn add_sol(&mut self, sol_amount: u64) {
        for asset in &mut self.numerator {
            if let Asset::SOL(existing_sol_amount) = asset {
                *existing_sol_amount += sol_amount;
                return;
            }
        }

        self.numerator.push(Asset::SOL(sol_amount));
    }

    pub(super) fn add_token(
        &mut self,
        token_mint: &Pubkey,
        token_pricing_source: Option<&TokenPricingSource>,
        token_amount: u64,
    ) {
        for asset in &mut self.numerator {
            if let Asset::Token(
                existing_token_mint,
                existing_token_pricing_source,
                existing_token_amount,
            ) = asset
            {
                if existing_token_mint != token_mint {
                    continue;
                }

                if existing_token_pricing_source.is_none() && token_pricing_source.is_some() {
                    *existing_token_pricing_source = token_pricing_source.cloned();
                }

                *existing_token_amount += token_amount;
                return;
            }
        }

        self.numerator.push(Asset::Token(
            *token_mint,
            token_pricing_source.cloned(),
            token_amount,
        ));
    }

    pub fn serialize_as_pod(&self, pod: &mut TokenValuePod) -> Result<()> {
        if self.numerator.len() > TOKEN_VALUE_MAX_NUMERATORS_SIZE {
            err!(ErrorCode::IndexOutOfBoundsException)?;
        }
        pod.num_numerator = self.numerator.len() as u64;
        for (numerator, asset) in pod.numerator.iter_mut().zip(&self.numerator) {
            asset.serialize_as_pod(numerator);
        }
        pod.denominator = self.denominator;

        Ok(())
    }
}

/// Pod type of `TokenValue`
#[zero_copy]
pub struct TokenValuePod {
    pub numerator: [AssetPod; TOKEN_VALUE_MAX_NUMERATORS_SIZE],
    pub num_numerator: u64,
    pub denominator: u64,
}

impl TokenValuePod {
    pub fn try_deserialize(&self) -> Result<TokenValue> {
        let mut numerator = Vec::with_capacity(self.num_numerator as usize);
        self.numerator[..self.num_numerator as usize]
            .iter()
            .try_for_each(|pod| {
                numerator.push(pod.try_deserialize()?);
                Ok::<_, Error>(())
            })?;

        Ok(TokenValue {
            numerator,
            denominator: self.denominator,
        })
    }

    pub fn get_asset_amount(&self, asset_mint: Option<&Pubkey>) -> u64 {
        self.numerator[..self.num_numerator as usize]
            .iter()
            .find_map(|asset| match (asset, asset_mint) {
                (
                    AssetPod {
                        discriminant: AssetPod::DISCRIMINANT_SOL,
                        sol_amount,
                        ..
                    },
                    None,
                ) => Some(*sol_amount),
                (
                    AssetPod {
                        discriminant: AssetPod::DISCRIMINANT_TOKEN,
                        token_mint,
                        token_amount,
                        ..
                    },
                    Some(supported_token_mint),
                ) if supported_token_mint == token_mint => Some(*token_amount),
                _ => None,
            })
            .unwrap_or_default()
    }

    pub(super) fn add_sol(&mut self, sol_amount: u64) {
        for asset in &mut self.numerator[..self.num_numerator as usize] {
            if let AssetPod {
                discriminant: AssetPod::DISCRIMINANT_SOL,
                sol_amount: existing_sol_amount,
                ..
            } = asset
            {
                *existing_sol_amount += sol_amount;
                return;
            }
        }

        Asset::SOL(sol_amount).serialize_as_pod(&mut self.numerator[self.num_numerator as usize]);
        self.num_numerator += 1;
    }

    pub(super) fn add_token(
        &mut self,
        token_mint: &Pubkey,
        token_pricing_source: Option<&TokenPricingSource>,
        token_amount: u64,
    ) {
        for asset in &mut self.numerator[..self.num_numerator as usize] {
            if let AssetPod {
                discriminant: AssetPod::DISCRIMINANT_TOKEN,
                token_mint: existing_token_mint,
                token_pricing_source: existing_token_pricing_source,
                token_amount: existing_token_amount,
                ..
            } = asset
            {
                if existing_token_mint != token_mint {
                    continue;
                }

                if existing_token_pricing_source.is_none() {
                    if let Some(token_pricing_source) = token_pricing_source {
                        token_pricing_source.serialize_as_pod(existing_token_pricing_source);
                    }
                }

                *existing_token_amount += token_amount;
                return;
            }
        }

        Asset::Token(*token_mint, token_pricing_source.cloned(), token_amount)
            .serialize_as_pod(&mut self.numerator[self.num_numerator as usize]);
        self.num_numerator += 1;
    }
}

#[derive(Clone, PartialEq, InitSpace, AnchorSerialize, AnchorDeserialize)]
#[cfg_attr(test, derive(Debug))]
pub enum Asset {
    // amount
    SOL(u64),
    // mint, known pricing source, amount
    Token(Pubkey, Option<TokenPricingSource>, u64),
}

impl Asset {
    pub fn serialize_as_pod(&self, pod: &mut AssetPod) {
        match self {
            Asset::SOL(sol_amount) => {
                pod.discriminant = AssetPod::DISCRIMINANT_SOL;
                pod.sol_amount = *sol_amount;
                pod.token_amount = 0;
                pod.token_mint = Pubkey::default();
                pod.token_pricing_source.set_none();
            }
            Asset::Token(token_mint, token_pricing_source, token_amount) => {
                pod.discriminant = AssetPod::DISCRIMINANT_TOKEN;
                pod.sol_amount = 0;
                pod.token_amount = *token_amount;
                pod.token_mint = *token_mint;
                if let Some(source) = token_pricing_source {
                    source.serialize_as_pod(&mut pod.token_pricing_source);
                } else {
                    pod.token_pricing_source.set_none();
                }
            }
        }
    }
}

/// Pod type of `Asset`
#[zero_copy]
pub struct AssetPod {
    pub discriminant: u8,
    _padding: [u8; 7],
    pub sol_amount: u64,
    pub token_amount: u64,
    pub token_mint: Pubkey,
    pub token_pricing_source: TokenPricingSourcePod,
}

impl AssetPod {
    pub(crate) const DISCRIMINANT_SOL: u8 = 1;
    pub(crate) const DISCRIMINANT_TOKEN: u8 = 2;

    pub fn try_deserialize(&self) -> Result<Asset> {
        Ok(match self.discriminant {
            Self::DISCRIMINANT_SOL => Asset::SOL(self.sol_amount),
            Self::DISCRIMINANT_TOKEN => Asset::Token(
                self.token_mint,
                self.token_pricing_source.try_deserialize()?,
                self.token_amount,
            ),
            _ => return Err(Error::from(ProgramError::InvalidAccountData)),
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

    impl TokenValueProvider for MockPricingSourceValueProvider<'_> {
        fn resolve_underlying_assets<'info>(
            self,
            _token_mint: &Pubkey,
            pricing_source_accounts: &[&'info AccountInfo<'info>],
            result: &mut TokenValue,
        ) -> Result<()> {
            require_eq!(pricing_source_accounts.len(), 0);

            result.numerator.clear();
            result.numerator.reserve_exact(self.numerator.len());

            result
                .numerator
                .extend(self.numerator.iter().map(|&asset| match asset {
                    MockAsset::SOL(sol_amount) => Asset::SOL(sol_amount),
                    MockAsset::Token(token_mint, token_amount) => {
                        Asset::Token(token_mint, None, token_amount)
                    }
                }));
            result.denominator = *self.denominator;

            Ok(())
        }
    }
}
