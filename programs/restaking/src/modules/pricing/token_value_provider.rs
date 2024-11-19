use crate::errors;
use crate::modules::pricing::TokenPricingSource;
use anchor_lang::prelude::*;

/// A type that can calculate the token amount as sol with its data.
pub trait TokenValueProvider {
    fn resolve_underlying_assets<'a, 'info: 'a>(
        token_pricing_source: &TokenPricingSource,
        pricing_source_accounts: Vec<&'a AccountInfo<'info>>,
    ) -> Result<TokenValue>;
}

/// a value representing total asset value of a pricing source.
#[derive(PartialEq, Debug)]
pub struct TokenValue {
    pub numerator: Vec<Asset>,
    pub denominator: u64,
}

impl std::fmt::Display for TokenValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq, Debug)]
pub enum Asset {
    // amount
    SOL(u64),
    // mint, known pricing source, amount
    TOKEN(Pubkey, Option<TokenPricingSource>, u64),
}

impl std::fmt::Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Copy, PartialEq)]
pub enum MockAsset {
    // amount
    SOL(u64),
    // mint, amount
    TOKEN(Pubkey, u64),
}

/// Example Mock Provider; Price: 1 denominated unit = 1.2 lamports
pub struct MockPricingSourceValueProvider;

#[cfg(test)]
impl TokenValueProvider for MockPricingSourceValueProvider {
    fn resolve_underlying_assets<'a, 'info: 'a>(
        token_pricing_source: &TokenPricingSource,
        pricing_source_accounts: Vec<&'a AccountInfo<'info>>,
    ) -> Result<TokenValue> {
        require_eq!(pricing_source_accounts.len(), 0);

        match token_pricing_source {
            TokenPricingSource::Mock {
                numerator,
                denominator,
            } => Ok(TokenValue {
                numerator: numerator
                    .iter()
                    .map(|asset| match asset {
                        MockAsset::SOL(amount) => Asset::SOL(*amount),
                        MockAsset::TOKEN(mint, amount) => Asset::TOKEN(*mint, None, *amount),
                    })
                    .collect(),
                denominator: *denominator,
            }),
            _ => Err(error!(
                errors::ErrorCode::TokenPricingSourceAccountNotFoundException
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::pricing::PricingService;

    #[test]
    fn test_mock_pricing_source() {
        let mut pricing_service = PricingService::new(&[]).unwrap();

        let mock_mint_10_10 = Pubkey::new_unique();
        pricing_service
            .resolve_token_pricing_source(
                &mock_mint_10_10,
                &TokenPricingSource::Mock {
                    numerator: vec![MockAsset::SOL(10)],
                    denominator: 10,
                },
            )
            .unwrap();
        assert_eq!(
            pricing_service
                .get_sol_amount_as_token(&mock_mint_10_10, 1_000)
                .unwrap(),
            1_000
        );
        assert_eq!(
            pricing_service
                .get_token_amount_as_sol(&mock_mint_10_10, 2_000)
                .unwrap(),
            2_000
        );

        let mock_mint_12_10 = Pubkey::new_unique();
        pricing_service
            .resolve_token_pricing_source(
                &mock_mint_12_10,
                &TokenPricingSource::Mock {
                    numerator: vec![
                        MockAsset::SOL(10_000),
                        MockAsset::TOKEN(mock_mint_10_10, 2_000),
                    ],
                    denominator: 10_000,
                },
            )
            .unwrap();
        assert_eq!(
            pricing_service
                .get_sol_amount_as_token(&mock_mint_12_10, 1_200)
                .unwrap(),
            1_000
        );
        assert_eq!(
            pricing_service
                .get_token_amount_as_sol(&mock_mint_12_10, 2_000)
                .unwrap(),
            2_400
        );

        let mock_mint_14_10 = Pubkey::new_unique();
        let mock_source_14_10 = &TokenPricingSource::Mock {
            numerator: vec![
                MockAsset::SOL(2_000),
                MockAsset::TOKEN(mock_mint_12_10, 10_000),
            ],
            denominator: 10_000,
        };
        pricing_service
            .resolve_token_pricing_source(&mock_mint_10_10, mock_source_14_10)
            .expect_err("resolve_token_pricing_source fails for already registered token");
        pricing_service
            .resolve_token_pricing_source(&mock_mint_14_10, mock_source_14_10)
            .unwrap();
        assert_eq!(
            pricing_service
                .get_sol_amount_as_token(&mock_mint_14_10, 1_400)
                .unwrap(),
            1_000
        );
        assert_eq!(
            pricing_service
                .get_token_amount_as_sol(&mock_mint_14_10, 2_000)
                .unwrap(),
            2_800
        );
    }
}
