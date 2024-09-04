use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::errors::ErrorCode;

const TOKEN_ALLOCATED_AMOUNT_RECORD_MAX_LEN: usize = 10;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct TokenAllocatedAmount {
    total_amount: u64,
    num_records: u8,
    _padding: [u8; 7],
    records: [TokenAllocatedAmountRecord; TOKEN_ALLOCATED_AMOUNT_RECORD_MAX_LEN],
}

impl TokenAllocatedAmount {
    /// Sum of contribution accrual rate (decimals = 2)
    /// e.g., rate = 135 => actual rate = 1.35
    pub fn total_contribution_accrual_rate(&self) -> Result<u64> {
        self.records.iter().try_fold(0u64, |sum, record| {
            sum.checked_add(record.total_contribution_accrual_rate()?)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
        })
    }

    pub fn add_total_amount(&mut self, amount: u64) -> Result<()> {
        self.total_amount = self
            .total_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }

    pub fn sub_total_amount(&mut self, amount: u64) -> Result<()> {
        self.total_amount = self
            .total_amount
            .checked_sub(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }

    pub fn allocate_new_record(&mut self) -> Result<&mut TokenAllocatedAmountRecord> {
        require_gt!(
            TOKEN_ALLOCATED_AMOUNT_RECORD_MAX_LEN,
            self.num_records as usize,
            ErrorCode::RewardExceededMaxTokenAllocatedAmountRecordException
        );

        let record = &mut self.records[self.num_records as usize];
        self.num_records += 1;

        Ok(record)
    }

    /// How to integrate multiple fields into a single array slice or whatever...
    /// You may change the return type if needed
    fn records_mut(&mut self) -> &mut [TokenAllocatedAmountRecord] {
        &mut self.records[..self.num_records as usize]
    }

    pub fn records_iter_mut(&mut self) -> impl Iterator<Item = &mut TokenAllocatedAmountRecord> {
        self.records_mut().iter_mut()
    }

    pub fn record_mut(
        &mut self,
        contribution_accrual_rate: u8,
    ) -> Option<&mut TokenAllocatedAmountRecord> {
        self.records_iter_mut()
            .find(|r| r.contribution_accrual_rate == contribution_accrual_rate)
    }

    pub fn sort_records(&mut self) {
        self.records_mut()
            .sort_by_key(|r| r.contribution_accrual_rate);
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct TokenAllocatedAmountRecord {
    amount: u64,
    /// Contribution accrual rate per 1 lamports (decimals = 2)
    /// e.g., rate = 135 => actual rate = 1.35
    contribution_accrual_rate: u8,
    _padding: [u8; 7],
}

impl TokenAllocatedAmountRecord {
    pub fn initialize(&mut self, contribution_accrual_rate: u8) {
        self.contribution_accrual_rate = contribution_accrual_rate;
        self.amount = 0;
    }

    pub fn amount(&self) -> u64 {
        self.amount
    }

    pub fn add_amount(&mut self, amount: u64) -> Result<()> {
        self.amount = self
            .amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        Ok(())
    }

    pub fn sub_amount(&mut self, amount: u64) -> Result<()> {
        self.amount = self
            .amount
            .checked_sub(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }

    pub fn contribution_accrual_rate(&self) -> u8 {
        self.contribution_accrual_rate
    }

    /// Contribution accrual rate multiplied by amount (decimals = 2)
    /// e.g., rate = 135 => actual rate = 1.35
    pub fn total_contribution_accrual_rate(&self) -> Result<u64> {
        self.amount
            .checked_mul(self.contribution_accrual_rate as u64)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }
}
