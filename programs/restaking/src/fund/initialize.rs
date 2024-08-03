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

impl FundV2 {
    pub(super) fn initialize(
        &mut self,
        default_protocol_fee_rate: u16,
        whitelisted_tokens: Vec<TokenInfo>,
        withdrawal_enabled_flag: bool,
        batch_processing_threshold_amount: u128,
        batch_processing_threshold_duration: i64,
    ) -> Result<()> {
        self.set_whitelisted_tokens(whitelisted_tokens)?;
        self.sol_amount_in = 0;
        self.withdrawal_status = Default::default();
        self.withdrawal_status.initialize(
            default_protocol_fee_rate,
            withdrawal_enabled_flag,
            batch_processing_threshold_amount,
            batch_processing_threshold_duration,
        )
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

impl WithdrawalStatus {
    fn initialize(
        &mut self,
        default_protocol_fee_rate: u16,
        withdrawal_enabled_flag: bool,
        batch_processing_threshold_amount: u128,
        batch_processing_threshold_duration: i64,
    ) -> Result<()> {
        self.set_default_protocol_fee_rate(default_protocol_fee_rate)?;
        self.set_withdrawal_enabled_flag(withdrawal_enabled_flag)?;
        self.set_batch_processing_threshold(
            batch_processing_threshold_amount,
            batch_processing_threshold_duration,
        )?;

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

    pub(super) fn set_withdrawal_enabled_flag(&mut self, flag: bool) -> Result<()> {
        self.withdrawal_enabled_flag = flag;

        Ok(())
    }

    pub(super) fn set_batch_processing_threshold(
        &mut self,
        amount: u128,
        duration: i64,
    ) -> Result<()> {
        // Threshold 값에 대한 validation 필요?
        self.batch_processing_threshold_amount = amount;
        self.batch_processing_threshold_duration = duration;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize() {
        let default_protocol_fee_rate = 100;
        let withdrawal_enabled_flag = true;
        let batch_processing_threshold_amount = 10;
        let batch_processing_threshold_duration = 10;

        let mut fund = FundV2 {
            whitelisted_tokens: vec![],
            sol_amount_in: 0,
            withdrawal_status: Default::default(),
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
            .initialize(
                default_protocol_fee_rate,
                tokens,
                withdrawal_enabled_flag,
                batch_processing_threshold_amount,
                batch_processing_threshold_duration,
            )
            .unwrap_err();

        let tokens = vec![token1, token2];
        fund.initialize(
            default_protocol_fee_rate,
            tokens.clone(),
            withdrawal_enabled_flag,
            batch_processing_threshold_amount,
            batch_processing_threshold_duration,
        )
        .unwrap();

        for (actual, expected) in std::iter::zip(fund.whitelisted_tokens, tokens) {
            assert_eq!(actual.address, expected.address);
            assert_eq!(actual.token_cap, expected.token_cap);
            assert_eq!(actual.token_amount_in, expected.token_amount_in);
        }
        assert_eq!(
            fund.withdrawal_status.default_protocol_fee_rate,
            default_protocol_fee_rate
        );
        assert_eq!(
            fund.withdrawal_status.withdrawal_enabled_flag,
            withdrawal_enabled_flag
        );
        assert_eq!(
            fund.withdrawal_status.batch_processing_threshold_amount,
            batch_processing_threshold_amount
        );
        assert_eq!(
            fund.withdrawal_status.batch_processing_threshold_duration,
            batch_processing_threshold_duration
        );
        assert_eq!(fund.withdrawal_status.pending_batch_withdrawal.batch_id, 1);
    }
}
