use std::collections::BTreeSet;

use anchor_lang::prelude::*;

use crate::{error::ErrorCode, fund::*};

impl Fund {
    pub(super) fn initialize(&mut self, admin: Pubkey, receipt_token_mint: Pubkey) -> Result<()> {
        self.admin = admin;
        self.receipt_token_mint = receipt_token_mint;
        self.sol_amount_in = 0;
        self.withdrawal_status = Default::default();

        Ok(())
    }

    pub(super) fn set_whitelisted_tokens(
        &mut self,
        whitelisted_tokens: Vec<TokenInfo>,
    ) -> Result<()> {
        Self::check_duplicates(&whitelisted_tokens)?;
        self.whitelisted_tokens = whitelisted_tokens;

        Ok(())
    }

    fn check_duplicates(tokens: &[TokenInfo]) -> Result<()> {
        let token_addresses: BTreeSet<_> = tokens.iter().map(|info| info.address).collect();
        if token_addresses.len() != tokens.len() {
            err!(ErrorCode::FundDuplicatedToken)?
        }

        Ok(())
    }
}

impl WithdrawalStatus {
    pub(super) fn set_sol_withdrawal_fee_rate(
        &mut self,
        sol_withdrawal_fee_rate: u16,
    ) -> Result<()> {
        // max protocol fee rate (상수) 넘어서지 못하게 하는 제약조건 필요?
        self.sol_withdrawal_fee_rate = sol_withdrawal_fee_rate;

        Ok(())
    }

    pub(super) fn set_withdrawal_enabled_flag(&mut self, flag: bool) -> Result<()> {
        self.withdrawal_enabled_flag = flag;

        Ok(())
    }

    pub(super) fn set_batch_processing_threshold(
        &mut self,
        amount: Option<u128>,
        duration: Option<i64>,
    ) -> Result<()> {
        // Threshold 값에 대한 validation 필요?
        if let Some(amount) = amount {
            self.batch_processing_threshold_amount = amount;
        }
        if let Some(duration) = duration {
            self.batch_processing_threshold_duration = duration;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize() {
        let admin = Pubkey::new_unique();
        let receipt_token_mint = Pubkey::new_unique();
        let sol_withdrawal_fee_rate = 100;
        let withdrawal_enabled_flag = false;
        let batch_processing_threshold_amount = 10;
        let batch_processing_threshold_duration = 10;

        let mut fund = Fund {
            admin: Pubkey::default(),
            receipt_token_mint: Pubkey::default(),
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

        fund.initialize(admin, receipt_token_mint).unwrap();

        let tokens = vec![token1, token2];
        fund.set_whitelisted_tokens(tokens.clone()).unwrap();

        fund.withdrawal_status
            .set_sol_withdrawal_fee_rate(sol_withdrawal_fee_rate)
            .unwrap();
        fund.withdrawal_status
            .set_withdrawal_enabled_flag(withdrawal_enabled_flag)
            .unwrap();
        fund.withdrawal_status
            .set_batch_processing_threshold(
                Some(batch_processing_threshold_amount),
                Some(batch_processing_threshold_duration),
            )
            .unwrap();

        assert_eq!(fund.admin, admin);
        assert_eq!(fund.receipt_token_mint, receipt_token_mint);
        for (actual, expected) in std::iter::zip(fund.whitelisted_tokens, tokens) {
            assert_eq!(actual.address, expected.address);
            assert_eq!(actual.token_cap, expected.token_cap);
            assert_eq!(actual.token_amount_in, expected.token_amount_in);
        }
        assert_eq!(
            fund.withdrawal_status.sol_withdrawal_fee_rate,
            sol_withdrawal_fee_rate
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
