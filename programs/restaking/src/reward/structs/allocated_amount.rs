use anchor_lang::prelude::*;

use crate::error::ErrorCode;

use super::*;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TokenAllocatedAmount {
    pub total_amount: u64,
    #[max_len(TOKEN_ALLOCATED_AMOUNT_RECORD_MAX_LEN)]
    pub records: Vec<TokenAllocatedAmountRecord>,
}

impl Default for TokenAllocatedAmount {
    fn default() -> Self {
        Self {
            records: vec![TokenAllocatedAmountRecord {
                contribution_accrual_rate: 100,
                amount: 0,
            }],
            total_amount: 0,
        }
    }
}

impl TokenAllocatedAmount {
    /// Sum of contribution accrual rate (decimals = 2)
    /// e.g., rate = 135 => actual rate = 1.35
    pub fn total_contribution_accrual_rate(&self) -> Result<u64> {
        self.records.iter().try_fold(0u64, |sum, record| {
            sum.checked_add(record.total_contribution_accrual_rate()?)
                .ok_or_else(|| error!(ErrorCode::CalculationFailure))
        })
    }
}

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TokenAllocatedAmountRecord {
    /// Contribution accrual rate per 1 lamports (decimals = 2)
    /// e.g., rate = 135 => actual rate = 1.35
    pub contribution_accrual_rate: u8,
    pub amount: u64,
}

impl TokenAllocatedAmountRecord {
    /// Contribution accrual rate multiplied by amount (decimals = 2)
    /// e.g., rate = 135 => actual rate = 1.35
    pub fn total_contribution_accrual_rate(&self) -> Result<u64> {
        self.amount
            .checked_mul(self.contribution_accrual_rate as u64)
            .ok_or_else(|| error!(ErrorCode::CalculationFailure))
    }
}
