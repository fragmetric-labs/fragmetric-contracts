use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::fund::*;

impl SupportedTokenInfo {
    pub fn set_capacity_amount(&mut self, capacity_amount: u64) -> Result<()> {
        if self.capacity_amount < self.accumulated_deposit_amount {
            err!(ErrorCode::FundInvalidUpdateError)?
        }
        self.capacity_amount = capacity_amount;

        Ok(())
    }
}

impl FundAccount {
    pub fn set_sol_capacity_amount(&mut self, capacity_amount: u64) -> Result<()> {
        if self.sol_capacity_amount < self.sol_accumulated_deposit_amount {
            err!(ErrorCode::FundInvalidUpdateError)?
        }
        self.sol_capacity_amount = capacity_amount;

        Ok(())
    }

    pub fn add_supported_token(
        &mut self,
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        capacity_amount: u64,
        pricing_source: TokenPricingSource,
        pricing_sources: &[AccountInfo],
    ) -> Result<()> {
        if self.supported_tokens.iter().any(|info| info.mint == mint) {
            err!(ErrorCode::FundAlreadySupportedTokenError)?
        }
        let token_info =
            SupportedTokenInfo::new(mint, program, decimals, capacity_amount, pricing_source);
        self.supported_tokens.push(token_info);
        self.update_token_prices(pricing_sources)?;

        Ok(())
    }
}

impl WithdrawalStatus {
    pub fn set_sol_withdrawal_fee_rate(&mut self, sol_withdrawal_fee_rate: u16) {
        self.sol_withdrawal_fee_rate = sol_withdrawal_fee_rate;
    }

    pub fn set_withdrawal_enabled_flag(&mut self, flag: bool) {
        self.withdrawal_enabled_flag = flag;
    }

    pub fn set_batch_processing_threshold(&mut self, amount: Option<u64>, duration: Option<i64>) {
        if let Some(amount) = amount {
            self.batch_processing_threshold_amount = amount;
        }
        if let Some(duration) = duration {
            self.batch_processing_threshold_duration = duration;
        }
    }
}

impl UserFundAccount {
    pub fn set_receipt_token_amount(&mut self, total_amount: u64) {
        self.receipt_token_amount = total_amount;
    }
}

#[cfg(test)]
mod tests {
    // use crate::constants::{BSOL_STAKE_POOL_ADDRESS, MSOL_STAKE_POOL_ADDRESS};
    use super::*;
    use crate::modules::fund::price::source;
    use crate::modules::fund::{FundAccount, SupportedTokenInfo, TokenPricingSource};

    #[test]
    fn test_update_fund() {
        let mut fund = FundAccount::new_uninitialized();
        fund.initialize_if_needed(0, Pubkey::new_unique());

        assert_eq!(fund.sol_capacity_amount, 0);
        assert_eq!(fund.withdrawal_status.sol_withdrawal_fee_rate, 0);
        assert!(fund.withdrawal_status.withdrawal_enabled_flag);
        assert_eq!(fund.withdrawal_status.batch_processing_threshold_amount, 0);
        assert_eq!(
            fund.withdrawal_status.batch_processing_threshold_duration,
            0
        );

        let new_sol_capacity_amount = 1_000_000_000 * 60_000;
        fund.set_sol_capacity_amount(new_sol_capacity_amount)
            .unwrap();
        assert_eq!(fund.sol_capacity_amount, new_sol_capacity_amount);

        let new_sol_withdrawal_fee_rate = 20;
        fund.withdrawal_status
            .set_sol_withdrawal_fee_rate(new_sol_withdrawal_fee_rate);
        assert_eq!(
            fund.withdrawal_status.sol_withdrawal_fee_rate,
            new_sol_withdrawal_fee_rate
        );

        fund.withdrawal_status.set_withdrawal_enabled_flag(false);
        assert!(!fund.withdrawal_status.withdrawal_enabled_flag);

        let new_amount = 10;
        let new_duration = 10;
        fund.withdrawal_status
            .set_batch_processing_threshold(Some(new_amount), None);
        assert_eq!(
            fund.withdrawal_status.batch_processing_threshold_amount,
            new_amount
        );
        assert_eq!(
            fund.withdrawal_status.batch_processing_threshold_duration,
            0
        );

        fund.withdrawal_status
            .set_batch_processing_threshold(None, Some(new_duration));
        assert_eq!(
            fund.withdrawal_status.batch_processing_threshold_amount,
            new_amount
        );
        assert_eq!(
            fund.withdrawal_status.batch_processing_threshold_duration,
            new_duration
        );
    }

    #[test]
    fn test_update_token() {
        let mut fund = FundAccount::new_uninitialized();
        fund.initialize_if_needed(0, Pubkey::new_unique());

        let token1 = SupportedTokenInfo {
            mint: Pubkey::new_unique(),
            program: Pubkey::new_unique(),
            decimals: 9,
            capacity_amount: 1_000_000_000 * 1000,
            accumulated_deposit_amount: 0,
            operation_reserved_amount: 1_000_000_000,
            price: 0,
            pricing_source: TokenPricingSource::SPLStakePool {
                address: Default::default(),
            },
            _reserved: [0; 128],
        };
        let token2 = SupportedTokenInfo {
            mint: Pubkey::new_unique(),
            program: Pubkey::new_unique(),
            decimals: 9,
            capacity_amount: 1_000_000_000 * 2000,
            accumulated_deposit_amount: 0,
            operation_reserved_amount: 2_000_000_000,
            price: 0,
            pricing_source: TokenPricingSource::MarinadeStakePool {
                address: Default::default(),
            },
            _reserved: [0; 128],
        };
        let mut dummy_lamports = 0u64;
        let mut dummy_data: [u8; 0] = [];
        let mut dummy_lamports2 = 0u64;
        let mut dummy_data2: [u8; 0] = [];
        let pricing_sources = &[
            AccountInfo::new(
                &source::SplStakePool::PROGRAM_ID,
                false,
                false,
                &mut dummy_lamports,
                &mut dummy_data,
                &source::SplStakePool::PROGRAM_ID,
                false,
                0,
            ),
            AccountInfo::new(
                &source::MarinadeStakePool::PROGRAM_ID,
                false,
                false,
                &mut dummy_lamports2,
                &mut dummy_data2,
                &source::MarinadeStakePool::PROGRAM_ID,
                false,
                0,
            ),
        ];

        fund.add_supported_token(
            token1.mint,
            token1.program,
            token2.decimals,
            token1.capacity_amount,
            token1.pricing_source,
            pricing_sources,
        )
        .unwrap();
        fund.add_supported_token(
            token2.mint,
            token2.program,
            token2.decimals,
            token2.capacity_amount,
            token2.pricing_source,
            pricing_sources,
        )
        .unwrap();
        assert_eq!(fund.supported_tokens.len(), 2);
        assert_eq!(
            fund.supported_tokens[0].capacity_amount,
            token1.capacity_amount
        );

        let new_token1_capacity_amount = 1_000_000_000 * 3000;
        fund.supported_token_mut(token1.mint)
            .unwrap()
            .set_capacity_amount(new_token1_capacity_amount)
            .unwrap();
        assert_eq!(
            fund.supported_tokens[0].capacity_amount,
            new_token1_capacity_amount
        );
    }
}
