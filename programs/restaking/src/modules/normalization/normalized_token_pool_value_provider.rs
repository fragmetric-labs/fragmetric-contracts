use anchor_lang::prelude::*;

use crate::modules::pricing::{Asset, TokenPricingSource, TokenValue, TokenValueProvider};

use super::*;

pub struct NormalizedTokenPoolValueProvider;

impl TokenValueProvider for NormalizedTokenPoolValueProvider {
    #[inline(never)]
    fn resolve_underlying_assets<'info>(
        self,
        token_mint: &Pubkey,
        pricing_source_accounts: &[&'info AccountInfo<'info>],
        result: &mut TokenValue,
    ) -> Result<()> {
        require_eq!(pricing_source_accounts.len(), 1);

        // let account = NormalizedTokenPoolService::deserialize_pool_account(pricing_source_accounts[0])?;
        let data = pricing_source_accounts[0].try_borrow_data()?;
        Self::resolve_from_buffer(token_mint, *data, result)
    }
}

impl NormalizedTokenPoolValueProvider {
    #[inline(always)]
    fn resolve_from_buffer(
        token_mint: &Pubkey,
        data: &[u8],
        result: &mut TokenValue,
    ) -> Result<()> {
        // skip 8 byte Anchor account discriminator
        let mut offset = 8;

        // skip data_version (u16) + bump (u8)
        offset += 2 + 1;

        // read normalized_token_mint
        let normalized_token_mint =
            Pubkey::new_from_array(data[offset..offset + 32].try_into().unwrap());
        offset += 32;

        require_keys_eq!(normalized_token_mint, *token_mint);

        // skip normalized_token_program (Pubkey)
        offset += 32;

        // parse supported_tokens vector length (u32 LE)
        let supported_tokens_length =
            u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        result.numerator.clear();
        result
            .numerator
            .reserve_exact(NormalizedTokenPoolAccount::MAX_SUPPORTED_TOKENS_SIZE);

        for _ in 0..supported_tokens_length {
            // parse mint
            let mint = Pubkey::new_from_array(data[offset..offset + 32].try_into().unwrap());
            offset += 32;

            // skip program (Pubkey) + reserve_account (Pubkey)
            offset += 32 + 32;

            // parse locked_amount
            let locked_amount = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
            offset += 8;

            // skip decimals (u8) + withdrawal_reserved_amount (u64) + one_token_as_sol (u64)
            offset += 1 + 8 + 8;

            // parse pricing_source: enum discriminator (u8) + Pubkey
            let pricing_source = TokenPricingSource::try_from_slice(&data[offset..offset + 33])?;
            offset += 33;

            result
                .numerator
                .push(Asset::Token(mint, Some(pricing_source), locked_amount));

            // skip reserved (u8 * 14)
            offset += 14;
        }

        // skip normalized_token_decimals (u8)
        offset += 1;

        // parse normalized_token_supply_amount (u64)
        let normalized_token_supply_amount =
            u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
        // offset += 8;

        result.denominator = normalized_token_supply_amount;

        Ok(())
    }

    #[cfg(test)]
    #[inline(never)]
    fn resolve_from_account<'info>(
        token_mint: &Pubkey,
        account: &NormalizedTokenPoolAccount,
        result: &mut TokenValue,
    ) -> Result<()> {
        require_keys_eq!(account.normalized_token_mint, *token_mint);

        result.numerator.clear();
        result
            .numerator
            .reserve_exact(NormalizedTokenPoolAccount::MAX_SUPPORTED_TOKENS_SIZE);

        result
            .numerator
            .extend(account.supported_tokens.iter().map(|supported_token| {
                Asset::Token(
                    supported_token.mint,
                    Some(supported_token.pricing_source.clone()),
                    supported_token.locked_amount,
                )
            }));
        result.denominator = account.normalized_token_supply_amount;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_resolve_from_buffer() {
        let mut account = NormalizedTokenPoolAccount {
            data_version: 1,
            bump: 255,
            normalized_token_mint: pubkey!("nSoLnkrvh2aY792pgCNT6hzx84vYtkviRzxvhf3ws8e"),
            normalized_token_program: pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
            supported_tokens: vec![
                NormalizedSupportedToken::new(
                    Pubkey::new_unique(),
                    pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
                    9,
                    Pubkey::new_unique(),
                    TokenPricingSource::SanctumSingleValidatorSPLStakePool {
                        address: Pubkey::new_unique(),
                    },
                ),
                NormalizedSupportedToken::new(
                    Pubkey::new_unique(),
                    pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
                    9,
                    Pubkey::new_unique(),
                    TokenPricingSource::SanctumSingleValidatorSPLStakePool {
                        address: Pubkey::new_unique(),
                    },
                ),
                NormalizedSupportedToken::new(
                    Pubkey::new_unique(),
                    pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
                    9,
                    Pubkey::new_unique(),
                    TokenPricingSource::SanctumSingleValidatorSPLStakePool {
                        address: Pubkey::new_unique(),
                    },
                ),
                NormalizedSupportedToken::new(
                    Pubkey::new_unique(),
                    pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
                    9,
                    Pubkey::new_unique(),
                    TokenPricingSource::SanctumSingleValidatorSPLStakePool {
                        address: Pubkey::new_unique(),
                    },
                ),
                NormalizedSupportedToken::new(
                    Pubkey::new_unique(),
                    pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
                    9,
                    Pubkey::new_unique(),
                    TokenPricingSource::SanctumSingleValidatorSPLStakePool {
                        address: Pubkey::new_unique(),
                    },
                ),
            ],

            normalized_token_decimals: 9,
            normalized_token_supply_amount: 0,
            normalized_token_value: TokenValue::default(),
            normalized_token_value_updated_slot: 0,
            one_normalized_token_as_sol: 1_000_000_000,
            _reserved: [0; 128],
        };

        for (index, supported_token) in account.supported_tokens.iter_mut().enumerate() {
            let amount = (index as u64 + 1) * 1_234_567_890;
            supported_token.lock_token(amount).unwrap();
            account.normalized_token_supply_amount += amount;
        }

        let mut buf = [0u8; 1298];
        account.try_serialize(&mut &mut buf[..]).unwrap();

        let mut result = TokenValue::default();
        NormalizedTokenPoolValueProvider::resolve_from_buffer(
            &account.normalized_token_mint,
            &buf,
            &mut result,
        )
        .unwrap();

        let mut result2 = TokenValue::default();
        NormalizedTokenPoolValueProvider::resolve_from_account(
            &account.normalized_token_mint,
            &account,
            &mut result2,
        )
        .unwrap();

        assert_eq!(result, result2);
    }
}
