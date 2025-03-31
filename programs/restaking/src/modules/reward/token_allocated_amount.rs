use anchor_lang::prelude::*;

use crate::errors::ErrorCode;

const REWARD_ACCOUNTS_TOKEN_ALLOCATED_AMOUNT_RECORD_MAX_LEN: usize = 10;
const MIN_CONTRIBUTION_ACCRUAL_RATE: u16 = 100;
const MAX_CONTRIBUTION_ACCRUAL_RATE: u16 = 500;

#[zero_copy]
#[repr(C)]
pub(super) struct TokenAllocatedAmount {
    total_amount: u64,
    num_records: u8,
    _padding: [u8; 7],
    records: [TokenAllocatedAmountRecord; REWARD_ACCOUNTS_TOKEN_ALLOCATED_AMOUNT_RECORD_MAX_LEN],
}

impl TokenAllocatedAmount {
    /// Sum of contribution accrual rate (decimals = 2)
    /// e.g., rate = 135 => actual rate = 1.35
    pub fn get_total_contribution_accrual_rate(&self) -> u64 {
        self.records.iter().fold(0, |sum, record| {
            sum + record.get_total_contribution_accrual_rate()
        })
    }

    #[inline(always)]
    fn get_records_iter_mut(&mut self) -> impl Iterator<Item = &mut TokenAllocatedAmountRecord> {
        self.records[..self.num_records as usize].iter_mut()
    }

    fn get_record_mut(
        &mut self,
        contribution_accrual_rate: u16,
    ) -> Option<&mut TokenAllocatedAmountRecord> {
        self.get_records_iter_mut()
            .find(|r| r.contribution_accrual_rate == contribution_accrual_rate)
    }

    pub fn update(
        &mut self,
        deltas: Vec<TokenAllocatedAmountDelta>,
    ) -> Result<Vec<TokenAllocatedAmountDelta>> {
        let total_amount_before: u64 = deltas.iter().map(|delta| delta.amount).sum();

        let mut effective_deltas = vec![];
        for delta in deltas {
            if delta.amount == 0 {
                continue;
            } else if delta.is_positive {
                effective_deltas.push(self.add(delta)?);
            } else {
                effective_deltas.extend(self.subtract(delta)?);
            }
        }

        // Accounting: check total amount before and after
        let total_amount: u64 = effective_deltas.iter().map(|delta| delta.amount).sum();

        require_eq!(
            total_amount_before,
            total_amount,
            ErrorCode::RewardInvalidAccountingException,
        );

        self.clear_stale_records();
        self.sort_records();

        Ok(effective_deltas)
    }

    /// When add amount, rate = null => rate = 1.0
    fn add(&mut self, mut delta: TokenAllocatedAmountDelta) -> Result<TokenAllocatedAmountDelta> {
        delta.assert_valid_addition()?;

        let contribution_accrual_rate = match delta.contribution_accrual_rate {
            Some(rate) => rate,
            None => {
                delta.contribution_accrual_rate = Some(100);
                100
            }
        };

        self.total_amount += delta.amount;
        if let Some(existing_record) = self.get_record_mut(contribution_accrual_rate) {
            existing_record.amount += delta.amount;
        } else {
            self.add_record(delta.amount, contribution_accrual_rate)?;
        }

        Ok(delta)
    }

    fn subtract<'a>(
        &'a mut self,
        mut delta: TokenAllocatedAmountDelta,
    ) -> Result<impl IntoIterator<Item = TokenAllocatedAmountDelta> + 'a> {
        delta.assert_valid_subtraction()?;

        self.total_amount -= delta.amount;
        Ok(match delta.contribution_accrual_rate {
            Some(rate) if rate > 100 => {
                let record = self
                    .get_record_mut(rate)
                    .ok_or_else(|| error!(ErrorCode::RewardInvalidAllocatedAmountDeltaException))?;
                record.amount -= delta.amount;
                OneOrManyDeltas::Single(delta)
            }
            _ => OneOrManyDeltas::Multiple(self.get_records_iter_mut().map_while(move |record| {
                if delta.amount == 0 {
                    return None;
                }

                let amount = record.amount.min(delta.amount);
                record.amount -= amount;
                delta.amount -= amount;
                Some(TokenAllocatedAmountDelta {
                    contribution_accrual_rate: Some(record.contribution_accrual_rate),
                    is_positive: false,
                    amount,
                })
            })),
        })
    }

    fn add_record(&mut self, amount: u64, contribution_accrual_rate: u16) -> Result<()> {
        require_gt!(
            REWARD_ACCOUNTS_TOKEN_ALLOCATED_AMOUNT_RECORD_MAX_LEN,
            self.num_records as usize,
            ErrorCode::RewardExceededMaxTokenAllocatedAmountRecordException
        );

        self.records[self.num_records as usize].initialize(amount, contribution_accrual_rate);
        self.num_records += 1;

        Ok(())
    }

    /// record is stale if amount = 0 and contribution accrual rate != 1.0.
    fn clear_stale_records(&mut self) {
        let mut l = 0;
        let mut r = self.num_records as usize;

        loop {
            // 1. move l to right until record[l] is stale
            // invariant: record[0..l] are all non-stale
            while l < self.num_records as usize && !self.records[l].is_stale() {
                l += 1;
            }
            // 2. move r to left until record[r-1] is non-stale
            // invariant: record[r..n] are all stale
            while r > 0 && self.records[r - 1].is_stale() {
                r -= 1;
            }

            if l == r {
                break; // done
            }

            // if l == n or r == 0, then obviously l == r
            // if l == r-1 then record[l], which is record[r-1] is both empty and non-empty so contradiction.
            // therefore l < r-1 so l+1 <= r-1.
            // swap record[l] and record[r-1]
            self.records.swap(l, r - 1);
            l += 1;
            r -= 1;
        }

        self.num_records = l as u8;
    }

    fn sort_records(&mut self) {
        self.records[..self.num_records as usize].sort_by_key(|r| r.contribution_accrual_rate);
    }
}

#[zero_copy]
#[repr(C)]
struct TokenAllocatedAmountRecord {
    amount: u64,
    /// Contribution accrual rate per 1 lamports (decimals = 2)
    /// e.g., rate = 135 => actual rate = 1.35
    contribution_accrual_rate: u16,
    _padding: [u8; 6],
}

impl TokenAllocatedAmountRecord {
    fn initialize(&mut self, amount: u64, contribution_accrual_rate: u16) {
        self.amount = amount;
        self.contribution_accrual_rate = contribution_accrual_rate;
    }

    /// Contribution accrual rate multiplied by amount (decimals = 2)
    /// e.g., rate = 135 => actual rate = 1.35
    #[inline(always)]
    fn get_total_contribution_accrual_rate(&self) -> u64 {
        self.amount * self.contribution_accrual_rate as u64
    }

    /// record is stale if amount = 0 and contribution accrual rate != 1.0.
    fn is_stale(&self) -> bool {
        self.amount == 0 && self.contribution_accrual_rate != MIN_CONTRIBUTION_ACCRUAL_RATE
    }
}

/// A change over [`TokenAllocatedAmount`].
#[derive(Clone)]
pub(super) struct TokenAllocatedAmountDelta {
    /// Contribution accrual rate per 1 lamports (decimals = 2)
    /// e.g., rate = 135 => actual rate = 1.35
    contribution_accrual_rate: Option<u16>,
    is_positive: bool,
    /// Nonzero - zero values are allowed but will be ignored
    amount: u64,
}

impl TokenAllocatedAmountDelta {
    pub fn new_positive(contribution_accrual_rate: Option<u16>, amount: u64) -> Self {
        Self {
            contribution_accrual_rate,
            is_positive: true,
            amount,
        }
    }

    pub fn new_negative(amount: u64) -> Self {
        Self {
            contribution_accrual_rate: None,
            is_positive: false,
            amount,
        }
    }

    fn assert_valid_contribution_accrual_rate(&self) -> Result<()> {
        if self.contribution_accrual_rate.as_ref().is_some_and(|rate| {
            !(MIN_CONTRIBUTION_ACCRUAL_RATE..MAX_CONTRIBUTION_ACCRUAL_RATE).contains(rate)
        }) {
            err!(ErrorCode::RewardInvalidAllocatedAmountDeltaException)?
        }

        Ok(())
    }

    fn assert_valid_addition(&self) -> Result<()> {
        self.assert_valid_contribution_accrual_rate()?;

        if !self.is_positive {
            err!(ErrorCode::RewardInvalidAllocatedAmountDeltaException)?;
        }

        Ok(())
    }

    fn assert_valid_subtraction(&self) -> Result<()> {
        self.assert_valid_contribution_accrual_rate()?;

        if self.is_positive {
            err!(ErrorCode::RewardInvalidAllocatedAmountDeltaException)?
        }

        Ok(())
    }
}

/// Auxillary type for better code - represents either a single delta or multiple deltas
enum OneOrManyDeltas<T: IntoIterator<Item = TokenAllocatedAmountDelta>> {
    Single(TokenAllocatedAmountDelta),
    Multiple(T),
}

enum OneOrManyDeltasIter<T: Iterator<Item = TokenAllocatedAmountDelta>> {
    Single(std::option::IntoIter<TokenAllocatedAmountDelta>),
    Multiple(T),
}

impl<T: IntoIterator<Item = TokenAllocatedAmountDelta>> IntoIterator for OneOrManyDeltas<T> {
    type Item = TokenAllocatedAmountDelta;
    type IntoIter = OneOrManyDeltasIter<T::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Single(single) => OneOrManyDeltasIter::Single(Some(single).into_iter()),
            Self::Multiple(multiple) => OneOrManyDeltasIter::Multiple(multiple.into_iter()),
        }
    }
}

impl<T: Iterator<Item = TokenAllocatedAmountDelta>> Iterator for OneOrManyDeltasIter<T> {
    type Item = TokenAllocatedAmountDelta;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Single(single) => single.next(),
            Self::Multiple(multiple) => multiple.next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytemuck::Zeroable;

    use super::*;

    #[test]
    fn test_clear_empty_records() {
        let non_empty = [0, 1, 3, 5];
        let mut amount = TokenAllocatedAmount::zeroed();
        amount.total_amount = 400;
        amount.num_records = 10;
        amount.records = std::array::from_fn(|i| {
            let mut record = TokenAllocatedAmountRecord::zeroed();
            record.contribution_accrual_rate = 100 + i as u16 * 10;
            record.amount = non_empty.contains(&i).then_some(100).unwrap_or_default();
            record
        });

        amount.clear_stale_records();
        amount.sort_records();

        assert_eq!(amount.num_records, 4);
        for (record, rate) in amount
            .get_records_iter_mut()
            .zip(non_empty.iter().map(|i| 100 + *i as u16 * 10))
        {
            assert_eq!(record.amount, 100);
            assert_eq!(record.contribution_accrual_rate, rate);
        }
    }

    #[test]
    fn test_subtract() {
        let deltas = vec![TokenAllocatedAmountDelta::new_negative(100)];
        let mut amount = TokenAllocatedAmount::zeroed();
        amount.total_amount = 150;
        amount.num_records = 2;
        amount.records[0].amount = 50;
        amount.records[0].contribution_accrual_rate = 100;
        amount.records[1].amount = 100;
        amount.records[1].contribution_accrual_rate = 120;

        let effective_deltas = amount.update(deltas).unwrap();

        assert_eq!(effective_deltas.len(), 2);
        assert_eq!(effective_deltas[0].amount, 50);
        assert_eq!(effective_deltas[0].contribution_accrual_rate, Some(100));
        assert_eq!(effective_deltas[1].amount, 50);
        assert_eq!(effective_deltas[1].contribution_accrual_rate, Some(120));

        // default(rate = 1.0) record is not removed
        assert_eq!(amount.num_records, 2);
        assert_eq!(amount.records[0].amount, 0);
        assert_eq!(amount.records[0].contribution_accrual_rate, 100);
        assert_eq!(amount.records[1].amount, 50);
        assert_eq!(amount.records[1].contribution_accrual_rate, 120);
    }
}
