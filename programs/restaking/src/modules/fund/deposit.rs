use crate::errors::ErrorCode;
use crate::modules::fund::{FundAccount, SupportedTokenInfo};
use anchor_lang::prelude::*;

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
