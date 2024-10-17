use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::fund::{FundAccount, SupportedTokenInfo};

impl SupportedTokenInfo {
    pub fn deposit_token(&mut self, amount: u64) -> Result<()> {
        let new_accumulated_deposit_amount = self
            .accumulated_deposit_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        if self.capacity_amount < new_accumulated_deposit_amount {
            err!(ErrorCode::FundExceededTokenCapacityAmountError)?
        }

        self.accumulated_deposit_amount = new_accumulated_deposit_amount;
        self.operation_reserved_amount = self
            .operation_reserved_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }
}

impl FundAccount {
    pub fn deposit_sol(&mut self, amount: u64) -> Result<()> {
        let new_sol_accumulated_deposit_amount = self
            .sol_accumulated_deposit_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        if self.sol_capacity_amount < new_sol_accumulated_deposit_amount {
            err!(ErrorCode::FundExceededSOLCapacityAmountError)?
        }

        self.sol_accumulated_deposit_amount = new_sol_accumulated_deposit_amount;
        self.sol_operation_reserved_amount = self
            .sol_operation_reserved_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DepositMetadata {
    pub wallet_provider: String,
    pub contribution_accrual_rate: u8, // 100 is 1.0
    pub expired_at: i64,
}

impl DepositMetadata {
    pub fn verify_expiration(&self) -> Result<()> {
        let current_timestamp = crate::utils::timestamp_now()?;

        if current_timestamp > self.expired_at {
            err!(ErrorCode::FundDepositMetadataSignatureExpiredError)?
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::modules::fund::price::source::*;

    use super::*;

    #[test]
    fn test_deposit_sol() {
        let mut fund = FundAccount::new_uninitialized();
        fund.initialize_if_needed(0, Pubkey::new_unique());
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
        let mut fund = FundAccount::new_uninitialized();
        fund.initialize_if_needed(0, Pubkey::new_unique());

        let mut dummy_lamports = 0u64;
        let mut dummy_data = [0u8; std::mem::size_of::<SplStakePool>()];
        let pricing_sources = &[SplStakePool::dummy_pricing_source_account_info(
            &mut dummy_lamports,
            &mut dummy_data,
        )];
        let token = SupportedTokenInfo::dummy_spl_stake_pool_token_info(pricing_sources[0].key());

        fund.add_supported_token(
            token.mint,
            token.program,
            token.decimals,
            1_000,
            token.pricing_source,
            pricing_sources,
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
