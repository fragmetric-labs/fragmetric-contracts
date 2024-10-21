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

    pub fn update(
        &mut self,
        deltas: Vec<TokenAllocatedAmountDelta>,
    ) -> Result<Vec<TokenAllocatedAmountDelta>> {
        let total_amount_orig = deltas.iter().try_fold(0u64, |sum, delta| {
            sum.checked_add(delta.amount)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
        })?;

        let mut effective_deltas = vec![];
        for delta in deltas.into_iter().filter(|delta| delta.amount > 0) {
            if delta.is_positive {
                effective_deltas.push(self.add(delta)?);
            } else {
                effective_deltas.extend(self.subtract(delta)?);
            }
        }

        // Accounting: check total amount before and after
        let total_amount_effective = effective_deltas.iter().try_fold(0u64, |sum, delta| {
            sum.checked_add(delta.amount)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
        })?;
        if total_amount_orig != total_amount_effective {
            err!(ErrorCode::RewardInvalidAccountingException)?
        }

        Ok(effective_deltas)
    }

    /// When add amount, rate = null => rate = 1.0
    fn add(&mut self, mut delta: TokenAllocatedAmountDelta) -> Result<TokenAllocatedAmountDelta> {
        delta.check_valid_addition()?;
        delta.set_default_contribution_accrual_rate();
        let contribution_accrual_rate = delta.contribution_accrual_rate.unwrap();

        if let Some(existing_record) = self.record_mut(contribution_accrual_rate) {
            existing_record.add_amount(delta.amount)?;
        } else {
            let record = self.allocate_new_record()?;
            record.initialize(contribution_accrual_rate);
            record.add_amount(delta.amount)?;
            self.sort_records();
        }
        self.add_total_amount(delta.amount)?;

        Ok(delta)
    }

    fn subtract(
        &mut self,
        delta: TokenAllocatedAmountDelta,
    ) -> Result<Vec<TokenAllocatedAmountDelta>> {
        delta.check_valid_subtraction()?;

        self.sub_total_amount(delta.amount)?;

        let mut deltas = vec![];
        if delta.contribution_accrual_rate.is_some_and(|r| r != 100) {
            let record = self
                .record_mut(delta.contribution_accrual_rate.unwrap())
                .ok_or_else(|| error!(ErrorCode::RewardInvalidAllocatedAmountDeltaException))?;
            record.sub_amount(delta.amount)?;
            deltas.push(delta);
        } else {
            let mut remaining_delta_amount = delta.amount;
            for record in self.records_iter_mut() {
                if remaining_delta_amount == 0 {
                    break;
                }

                let amount = std::cmp::min(record.amount(), remaining_delta_amount);
                if amount > 0 {
                    record.sub_amount(amount).unwrap();
                    remaining_delta_amount -= amount;
                    deltas.push(TokenAllocatedAmountDelta {
                        contribution_accrual_rate: Some(record.contribution_accrual_rate()),
                        is_positive: false,
                        amount,
                    });
                }
            }
        }

        Ok(deltas)
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

/// A change over [`TokenAllocatedAmount`].
pub struct TokenAllocatedAmountDelta {
    /// Contribution accrual rate per 1 lamports (decimals = 2)
    /// e.g., rate = 135 => actual rate = 1.35
    pub contribution_accrual_rate: Option<u8>,
    pub is_positive: bool,
    /// Nonzero - zero values are allowed but will be ignored
    pub amount: u64,
}

impl TokenAllocatedAmountDelta {
    fn check_valid_addition(&self) -> Result<()> {
        let is_contribution_accrual_rate_invalid = || {
            self.contribution_accrual_rate
                .is_some_and(|rate| !(100..200).contains(&rate))
        };
        if !self.is_positive || is_contribution_accrual_rate_invalid() {
            err!(ErrorCode::RewardInvalidAllocatedAmountDeltaException)?
        }

        Ok(())
    }

    fn check_valid_subtraction(&self) -> Result<()> {
        if self.is_positive {
            err!(ErrorCode::RewardInvalidAllocatedAmountDeltaException)?
        }

        Ok(())
    }

    fn set_default_contribution_accrual_rate(&mut self) {
        if self.contribution_accrual_rate.is_none() {
            self.contribution_accrual_rate = Some(100);
        }
    }
}
