use crate::modules::pricing::{
    Asset, TokenPricingSource, TokenPricingSourcePod, TokenValue, TokenValueProvider,
};
use anchor_lang::prelude::*;
use bytemuck::Zeroable;

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

        let data = pricing_source_accounts[0].try_borrow_data()?;

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
}
