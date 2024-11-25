use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::pricing::TokenPricingSource;

use super::FundAccount;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct SupportedToken {
    pub(super) mint: Pubkey,
    pub(super) program: Pubkey,
    pub(super) decimals: u8,
    capacity_amount: u64,
    accumulated_deposit_amount: u64,
    pub(super) operation_reserved_amount: u64,
    pub(super) one_token_as_sol: u64,
    pub(super) pricing_source: TokenPricingSource,
    pub(super) operating_amount: u64,
    _reserved: [u8; 120],
}

impl SupportedToken {
    pub(super) fn new(
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        capacity_amount: u64,
        pricing_source: TokenPricingSource,
    ) -> Self {
        Self {
            mint,
            program,
            decimals,
            capacity_amount,
            accumulated_deposit_amount: 0,
            operation_reserved_amount: 0,
            one_token_as_sol: 0,
            pricing_source,
            operating_amount: 0,
            _reserved: [0; 120],
        }
    }

    // TODO v0.3/operation: visibility
    pub(in crate::modules) fn get_operation_reserved_amount(&self) -> u64 {
        self.operation_reserved_amount
    }

    // TODO v0.3/operation: visibility
    pub(in crate::modules) fn set_operation_reserved_amount(&mut self, amount: u64) {
        self.operation_reserved_amount = amount;
    }

    // TODO v0.3/operation: visibility
    pub(in crate::modules) fn get_operating_amount(&self) -> u64 {
        self.operating_amount
    }

    // TODO v0.3/operation: visibility
    pub(in crate::modules) fn set_operating_amount(&mut self, amount: u64) {
        self.operating_amount = amount;
    }

    pub(super) fn set_capacity_amount(&mut self, capacity_amount: u64) -> anchor_lang::Result<()> {
        require_gte!(
            capacity_amount,
            self.accumulated_deposit_amount,
            ErrorCode::FundInvalidUpdateError
        );

        self.capacity_amount = capacity_amount;

        Ok(())
    }

    pub(super) fn deposit_token(&mut self, amount: u64) -> anchor_lang::Result<()> {
        let new_accumulated_deposit_amount = self
            .accumulated_deposit_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        require_gte!(
            self.capacity_amount,
            new_accumulated_deposit_amount,
            ErrorCode::FundExceededTokenCapacityAmountError
        );

        self.accumulated_deposit_amount = new_accumulated_deposit_amount;
        self.operation_reserved_amount = self
            .operation_reserved_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::pricing::TokenPricingSource;
    use anchor_lang::{AccountDeserialize, Space};

    fn create_initialized_fund_account() -> FundAccount {
        let buffer = [0u8; 8 + FundAccount::INIT_SPACE];
        let mut fund = FundAccount::try_deserialize_unchecked(&mut &buffer[..]).unwrap();
        fund.update(0, Pubkey::new_unique(), 9, 0);
        fund
    }

    #[test]
    fn test_initialize_update_fund_account() {
        let mut fund = create_initialized_fund_account();

        assert_eq!(fund.sol_capacity_amount, 0);
        assert_eq!(fund.withdrawal.get_sol_withdrawal_fee_rate_as_f32(), 0.);
        assert!(fund.withdrawal.withdrawal_enabled_flag);
        assert_eq!(fund.withdrawal.batch_processing_threshold_amount, 0);
        assert_eq!(fund.withdrawal.batch_processing_threshold_duration, 0);

        fund.sol_accumulated_deposit_amount = 1_000_000_000_000;
        fund.set_sol_capacity_amount(0).unwrap_err();

        let new_amount = 10;
        let new_duration = 10;
        fund.withdrawal
            .set_batch_processing_threshold(Some(new_amount), None);
        assert_eq!(
            fund.withdrawal.batch_processing_threshold_amount,
            new_amount
        );
        assert_eq!(fund.withdrawal.batch_processing_threshold_duration, 0);

        fund.withdrawal
            .set_batch_processing_threshold(None, Some(new_duration));
        assert_eq!(
            fund.withdrawal.batch_processing_threshold_amount,
            new_amount
        );
        assert_eq!(
            fund.withdrawal.batch_processing_threshold_duration,
            new_duration
        );
    }

    #[test]
    fn test_update_token() {
        let mut fund = create_initialized_fund_account();

        let token1 = Pubkey::new_unique();
        let token2 = Pubkey::new_unique();

        fund.add_supported_token(
            token1,
            Pubkey::default(),
            9,
            1_000_000_000,
            TokenPricingSource::SPLStakePool {
                address: Pubkey::new_unique(),
            },
        )
        .unwrap();
        fund.add_supported_token(
            token2,
            Pubkey::default(),
            9,
            1_000_000_000,
            TokenPricingSource::MarinadeStakePool {
                address: Pubkey::new_unique(),
            },
        )
        .unwrap();
        fund.add_supported_token(
            token1,
            Pubkey::default(),
            9,
            1_000_000_000,
            TokenPricingSource::MarinadeStakePool {
                address: Pubkey::new_unique(),
            },
        )
        .unwrap_err();
        assert_eq!(fund.supported_tokens.len(), 2);
        assert_eq!(fund.supported_tokens[0].capacity_amount, 1_000_000_000);

        fund.supported_tokens[0].accumulated_deposit_amount = 1_000_000_000;
        fund.get_supported_token_mut(&token1)
            .unwrap()
            .set_capacity_amount(0)
            .unwrap_err();
    }

    #[test]
    fn test_deposit_sol() {
        let mut fund = create_initialized_fund_account();
        fund.set_sol_capacity_amount(100_000).unwrap();

        assert_eq!(fund.sol_operation_reserved_amount, 0);
        assert_eq!(fund.sol_accumulated_deposit_amount, 0);

        fund.deposit_sol(100_000).unwrap();
        assert_eq!(fund.sol_operation_reserved_amount, 100_000);
        assert_eq!(fund.sol_accumulated_deposit_amount, 100_000);

        fund.deposit_sol(100_000).unwrap_err();
    }

    #[test]
    fn test_deposit_token() {
        let mut fund = create_initialized_fund_account();

        fund.add_supported_token(
            Pubkey::new_unique(),
            Pubkey::default(),
            9,
            1_000,
            TokenPricingSource::SPLStakePool {
                address: Pubkey::new_unique(),
            },
        )
        .unwrap();

        assert_eq!(fund.supported_tokens[0].operation_reserved_amount, 0);
        assert_eq!(fund.supported_tokens[0].accumulated_deposit_amount, 0);

        fund.supported_tokens[0].deposit_token(1_000).unwrap();
        assert_eq!(fund.supported_tokens[0].operation_reserved_amount, 1_000);
        assert_eq!(fund.supported_tokens[0].accumulated_deposit_amount, 1_000);

        fund.supported_tokens[0].deposit_token(1_000).unwrap_err();
    }
}
