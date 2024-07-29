use std::collections::BTreeSet;

use anchor_lang::prelude::*;

use crate::{error::ErrorCode, fund::*};

impl Fund {
    pub(super) fn initialize(&mut self, admin: Pubkey, receipt_token_mint: Pubkey) -> Result<()> {
        self.admin = admin;
        self.receipt_token_mint = receipt_token_mint;
        Ok(())
    }
}

impl FundV1 {
    pub(super) fn initialize(
        &mut self,
        default_protocol_fee_rate: u16,
        whitelisted_tokens: Vec<TokenInfo>,
    ) -> Result<()> {
        self.set_default_protocol_fee_rate(default_protocol_fee_rate)?;
        self.set_whitelisted_tokens(whitelisted_tokens)?;
        self.sol_amount_in = 0;

        Ok(())
    }

    pub(super) fn set_default_protocol_fee_rate(
        &mut self,
        default_protocol_fee_rate: u16,
    ) -> Result<()> {
        // max protocol fee rate (상수) 넘어서지 못하게 하는 제약조건 필요?
        self.default_protocol_fee_rate = default_protocol_fee_rate;

        Ok(())
    }

    fn set_whitelisted_tokens(&mut self, whitelisted_tokens: Vec<TokenInfo>) -> Result<()> {
        Self::check_duplicates(&whitelisted_tokens)?;
        self.whitelisted_tokens = whitelisted_tokens;

        Ok(())
    }

    fn check_duplicates(tokens: &[TokenInfo]) -> Result<()> {
        let token_addresses: BTreeSet<_> = tokens.iter().map(|info| info.address).collect();
        if token_addresses.len() != tokens.len() {
            return Err(ErrorCode::FundDuplicatedToken)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize() {
        let default_protocol_fee_rate = 100;

        let mut fund = FundV1 {
            default_protocol_fee_rate: 0,
            whitelisted_tokens: vec![],
            sol_amount_in: 0,
        };

        let token1 = TokenInfo {
            address: Pubkey::new_unique(),
            token_cap: 1_000_000_000 * 1000,
            token_amount_in: 1_000_000_000,
        };
        let token2 = TokenInfo {
            address: Pubkey::new_unique(),
            token_cap: 1_000_000_000 * 2000,
            token_amount_in: 2_000_000_000,
        };

        let tokens = vec![token1.clone(), token1.clone()];
        let _ = fund
            .initialize(default_protocol_fee_rate, tokens)
            .unwrap_err();

        let tokens = vec![token1, token2];
        fund.initialize(default_protocol_fee_rate, tokens.clone())
            .unwrap();

        assert_eq!(fund.default_protocol_fee_rate, default_protocol_fee_rate);
        for (actual, expected) in std::iter::zip(fund.whitelisted_tokens, tokens) {
            assert_eq!(actual.address, expected.address);
            assert_eq!(actual.token_cap, expected.token_cap);
            assert_eq!(actual.token_amount_in, expected.token_amount_in);
        }
    }
}
