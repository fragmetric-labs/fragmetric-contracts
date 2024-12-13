use anchor_lang::prelude::*;

use crate::errors;
use crate::utils;

#[derive(Clone, Copy, Default, Debug)]
pub struct WeightedAllocationParticipant {
    pub weight: u64,
    pub allocated_amount: u64,
    pub capacity_amount: u64,
    last_delta_amount: i128,
}

impl WeightedAllocationParticipant {
    pub fn new(weight: u64, allocated_amount: u64, capacity_amount: u64) -> Self {
        Self {
            weight,
            allocated_amount,
            capacity_amount,
            last_delta_amount: 0,
        }
    }

    fn is_full(&self) -> bool {
        self.allocated_amount >= self.capacity_amount
    }

    pub fn get_last_put_amount(&self) -> Result<u64> {
        u64::try_from(self.last_delta_amount)
            .map_err(|_| error!(errors::ErrorCode::CalculationArithmeticException))
    }

    pub fn get_last_cut_amount(&self) -> Result<u64> {
        u64::try_from(-self.last_delta_amount)
            .map_err(|_| error!(errors::ErrorCode::CalculationArithmeticException))
    }
}

pub struct WeightedAllocationStrategy<const N: usize> {
    pub participants: [WeightedAllocationParticipant; N],
}

impl<const N: usize> WeightedAllocationStrategy<N> {
    pub fn new(participants: impl IntoIterator<Item = WeightedAllocationParticipant>) -> Self {
        let mut strategy = Self {
            participants: [WeightedAllocationParticipant::default(); N],
        };
        for (i, participant) in participants.into_iter().enumerate() {
            strategy.participants[i] = participant;
        }
        strategy
    }

    /// returns remaining_amount after the allocation made
    pub fn put(&mut self, amount: u64) -> Result<u64> {
        let mut remaining_amount = amount;

        // remember original amount
        self.participants.iter_mut().for_each(|participant| {
            participant.last_delta_amount = participant.allocated_amount as i128;
        });


        while remaining_amount > 0 {
            let mut target_participants_count = 0;
            let mut target_participants_index: [usize; N] = [0; N];
            let mut target_participant_max_allocated_amount: u64 = 0;
            let mut basis_participant_index: usize = 0;

            for (i, p) in self.participants.iter().enumerate() {
                if p.weight > 0 && !p.is_full() {
                    target_participants_index[target_participants_count] = i;
                    target_participants_count += 1;

                    // find the basis participant who has max allocated amount
                    if p.allocated_amount > target_participant_max_allocated_amount {
                        target_participant_max_allocated_amount = p.allocated_amount;
                        basis_participant_index = i;
                    }
                }
            }

            if target_participants_count == 0 {
                break;
            }

            // calculate shortages
            let basis_participant = &self.participants[basis_participant_index];
            let mut shortage_amounts: [u64; N] = [0; N];
            for i in &target_participants_index[..target_participants_count] {
                let p = &self.participants[*i];
                let target_amount = utils::get_proportional_amount(
                    basis_participant.allocated_amount,
                    p.weight,
                    basis_participant.weight,
                )
                .ok_or_else(|| error!(errors::ErrorCode::CalculationArithmeticException))?;
                shortage_amounts[*i] = target_amount.saturating_sub(p.allocated_amount);
            }

            // first, allocate remaining resources proportionally relative to each shortages
            let total_shortage_amount = shortage_amounts[..target_participants_count].iter().sum::<u64>();
            if total_shortage_amount > 0 {
                let total_allocatable_amount = remaining_amount.min(total_shortage_amount);
                let mut allocated_amount = 0;
                for (i, shortage) in shortage_amounts[..target_participants_count].iter().enumerate() {
                    if *shortage == 0 {
                        continue;
                    }
                    let p = &mut self.participants[target_participants_index[i]];
                    let allocatable_amount = p.capacity_amount - p.allocated_amount;
                    let allocating_amount = utils::get_proportional_amount(
                        total_allocatable_amount,
                        *shortage,
                        total_shortage_amount,
                    )
                    .ok_or_else(|| error!(errors::ErrorCode::CalculationArithmeticException))?
                    .min(allocatable_amount);
                    p.allocated_amount += allocating_amount;
                    allocated_amount += allocating_amount;
                }

                if allocated_amount > 0 {
                    remaining_amount -= allocated_amount;

                    // restart allocation
                    continue;
                }
            }

            // then, allocate remaining resources proportionally relative to each weights
            let total_weight = target_participants_index[..target_participants_count]
                .into_iter()
                .map(|i| self.participants[*i].weight)
                .sum();
            let mut allocated_amount = 0;
            for i in &target_participants_index[..target_participants_count] {
                let p = &mut self.participants[*i];
                let allocatable_amount = p.capacity_amount.saturating_sub(p.allocated_amount);
                let allocating_amount =
                    utils::get_proportional_amount(remaining_amount, p.weight, total_weight)
                        .ok_or_else(|| error!(errors::ErrorCode::CalculationArithmeticException))?
                        .min(allocatable_amount);
                p.allocated_amount += allocating_amount;
                allocated_amount += allocating_amount;
            }

            if remaining_amount == 1 {
                // cannot allocate more due to precision
                let max_weighted_target_participant_index = {
                    target_participants_index[..target_participants_count]
                        .iter()
                        .filter(|i| !self.participants[**i].is_full())
                        .max_by_key(|i| self.participants[**i].weight)
                };
                if let Some(i) = max_weighted_target_participant_index {
                    self.participants[*i].allocated_amount += 1;
                    remaining_amount = 0;
                }
                break;
            } else if allocated_amount == 0 {
                // cannot allocate more due to maxed capacity
                break;
            }

            remaining_amount -= allocated_amount;
        }


        // set delta amount
        self.participants.iter_mut().for_each(|p| {
            p.last_delta_amount = (p.allocated_amount as i128) - p.last_delta_amount;
        });

        crate::utils::debug_msg_heap_size("strategy...9");
        Ok(remaining_amount)
    }

    /// returns required_amount after the de-allocation made
    fn cut(&mut self, amount: u64) -> Result<u64> {
        let mut required_amount = amount;

        // remember original amount
        self.participants.iter_mut().for_each(|p| {
            p.last_delta_amount = p.allocated_amount as i128;
        });

        // cut from non-zero weighted participants first
        {
            let mut target_participants_count = 0;
            let mut target_participants_index_weight: [(usize, u64); N] = [(0, 0); N];
            for (i, p) in self.participants.iter().enumerate() {
                if p.weight > 0 {
                    target_participants_index_weight[target_participants_count] = (i, p.weight);
                    target_participants_count += 1;
                }
            }

            // cut by lowest weighted participant
            target_participants_index_weight[..target_participants_count].sort_by(|(_, a), (_, b)| a.cmp(b));

            for (i, _) in &target_participants_index_weight[..target_participants_count] {
                if required_amount == 0 {
                    break;
                }
                let p = &mut self.participants[*i];
                let deallocating_amount = required_amount.min(p.allocated_amount);
                p.allocated_amount -= deallocating_amount;
                required_amount -= deallocating_amount;
            }
        }

        // cut from zero weighted participants if needed
        if required_amount > 0 {
            let mut target_participants_count = 0;
            let mut target_participants_index: [usize; N] = [0; N];
            for (i, p) in self.participants.iter().enumerate() {
                if p.weight == 0 {
                    target_participants_index[target_participants_count] = i;
                    target_participants_count += 1;
                }
            }

            let total_allocated_amount = target_participants_index[..target_participants_count]
                .iter()
                .map(|i| self.participants[*i].allocated_amount)
                .sum();

            let mut deallocated_amount = 0;
            for i in &target_participants_index[..target_participants_count] {
                let p = &mut self.participants[*i];
                let deallocating_amount = utils::get_proportional_amount(
                    required_amount,
                    p.allocated_amount,
                    total_allocated_amount,
                )
                .ok_or_else(|| error!(errors::ErrorCode::CalculationArithmeticException))?
                .min(p.allocated_amount);
                p.allocated_amount -= deallocating_amount;
                deallocated_amount += deallocating_amount;
            }
            required_amount -= deallocated_amount;
        }

        // set delta amount
        self.participants.iter_mut().for_each(|p| {
            p.last_delta_amount = (p.allocated_amount as i128) - p.last_delta_amount;
        });

        Ok(required_amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_general_scenario() {
        let mut strategy = WeightedAllocationStrategy::<4>::new([
            WeightedAllocationParticipant::new(4, 0, 2000),
            WeightedAllocationParticipant::new(2, 0, u64::MAX),
            WeightedAllocationParticipant::new(1, 0, u64::MAX),
            WeightedAllocationParticipant::new(0, 0, u64::MAX),
        ]);

        // step-by-step allocations and cuts
        strategy.participants[3].allocated_amount = 100;
        assert_eq!(strategy.participants[3].allocated_amount, 100);

        assert_eq!(strategy.put(700).unwrap(), 0);
        assert_eq!(strategy.participants[0].allocated_amount, 400);
        assert_eq!(strategy.participants[1].allocated_amount, 200);
        assert_eq!(strategy.participants[2].allocated_amount, 100);
        assert_eq!(strategy.participants[3].allocated_amount, 100);
        assert_eq!(strategy.participants[0].last_delta_amount, 400);
        assert_eq!(strategy.participants[1].last_delta_amount, 200);
        assert_eq!(strategy.participants[2].last_delta_amount, 100);
        assert_eq!(strategy.participants[3].last_delta_amount, 0);
        assert_eq!(strategy.participants[0].get_last_put_amount().unwrap(), 400);
        assert_eq!(strategy.participants[1].get_last_put_amount().unwrap(), 200);
        assert_eq!(strategy.participants[2].get_last_put_amount().unwrap(), 100);
        assert_eq!(strategy.participants[3].get_last_put_amount().unwrap(), 0);
        strategy.participants[0].get_last_cut_amount().unwrap_err();
        strategy.participants[1].get_last_cut_amount().unwrap_err();
        strategy.participants[2].get_last_cut_amount().unwrap_err();
        assert_eq!(strategy.participants[3].get_last_cut_amount().unwrap(), 0);

        assert_eq!(strategy.put(700).unwrap(), 0);
        assert_eq!(strategy.participants[0].allocated_amount, 800);
        assert_eq!(strategy.participants[1].allocated_amount, 400);
        assert_eq!(strategy.participants[2].allocated_amount, 200);
        assert_eq!(strategy.participants[3].allocated_amount, 100);
        assert_eq!(strategy.participants[0].last_delta_amount, 400);
        assert_eq!(strategy.participants[1].last_delta_amount, 200);
        assert_eq!(strategy.participants[2].last_delta_amount, 100);
        assert_eq!(strategy.participants[3].last_delta_amount, 0);

        // direct allocation: put 400 to jitoSOL
        strategy.participants[0].allocated_amount += 400;
        assert_eq!(strategy.participants[0].allocated_amount, 1200);
        assert_eq!(strategy.participants[1].allocated_amount, 400);
        assert_eq!(strategy.participants[2].allocated_amount, 200);
        assert_eq!(strategy.participants[3].allocated_amount, 100);

        // direct allocation: put 50 to bSOL
        strategy.participants[2].allocated_amount += 50;
        assert_eq!(strategy.participants[0].allocated_amount, 1200);
        assert_eq!(strategy.participants[1].allocated_amount, 400);
        assert_eq!(strategy.participants[2].allocated_amount, 250);
        assert_eq!(strategy.participants[3].allocated_amount, 100);

        assert_eq!(strategy.put(320).unwrap(), 0);
        assert_eq!(strategy.participants[0].allocated_amount, 1240);
        assert_eq!(strategy.participants[1].allocated_amount, 620);
        assert_eq!(strategy.participants[2].allocated_amount, 310);
        assert_eq!(strategy.participants[3].allocated_amount, 100);
        assert_eq!(strategy.participants[0].last_delta_amount, 40);
        assert_eq!(strategy.participants[1].last_delta_amount, 220);
        assert_eq!(strategy.participants[2].last_delta_amount, 60);
        assert_eq!(strategy.participants[3].last_delta_amount, 0);

        assert_eq!(strategy.cut(300).unwrap(), 0);
        assert_eq!(strategy.participants[0].allocated_amount, 1240);
        assert_eq!(strategy.participants[1].allocated_amount, 620);
        assert_eq!(strategy.participants[2].allocated_amount, 10);
        assert_eq!(strategy.participants[3].allocated_amount, 100);
        assert_eq!(strategy.participants[0].last_delta_amount, 0);
        assert_eq!(strategy.participants[1].last_delta_amount, 0);
        assert_eq!(strategy.participants[2].last_delta_amount, -300);
        assert_eq!(strategy.participants[3].last_delta_amount, 0);
        assert_eq!(strategy.participants[0].get_last_cut_amount().unwrap(), 0);
        assert_eq!(strategy.participants[1].get_last_cut_amount().unwrap(), 0);
        assert_eq!(strategy.participants[2].get_last_cut_amount().unwrap(), 300);
        assert_eq!(strategy.participants[3].get_last_cut_amount().unwrap(), 0);
        assert_eq!(strategy.participants[0].get_last_put_amount().unwrap(), 0);
        assert_eq!(strategy.participants[1].get_last_put_amount().unwrap(), 0);
        strategy.participants[2].get_last_put_amount().unwrap_err();
        assert_eq!(strategy.participants[3].get_last_put_amount().unwrap(), 0);

        assert_eq!(strategy.put(370).unwrap(), 0);
        assert_eq!(strategy.participants[0].allocated_amount, 1280);
        assert_eq!(strategy.participants[1].allocated_amount, 640);
        assert_eq!(strategy.participants[2].allocated_amount, 320);
        assert_eq!(strategy.participants[3].allocated_amount, 100);
        assert_eq!(strategy.participants[0].last_delta_amount, 40);
        assert_eq!(strategy.participants[1].last_delta_amount, 20);
        assert_eq!(strategy.participants[2].last_delta_amount, 310);
        assert_eq!(strategy.participants[3].last_delta_amount, 0);

        assert_eq!(strategy.cut(640).unwrap(), 0);
        assert_eq!(strategy.participants[0].allocated_amount, 1280);
        assert_eq!(strategy.participants[1].allocated_amount, 320);
        assert_eq!(strategy.participants[2].allocated_amount, 0);
        assert_eq!(strategy.participants[3].allocated_amount, 100);
        assert_eq!(strategy.participants[0].last_delta_amount, 0);
        assert_eq!(strategy.participants[1].last_delta_amount, -320);
        assert_eq!(strategy.participants[2].last_delta_amount, -320);
        assert_eq!(strategy.participants[3].last_delta_amount, 0);

        assert_eq!(strategy.put(480).unwrap(), 0);
        assert_eq!(strategy.participants[0].allocated_amount, 1280);
        assert_eq!(strategy.participants[1].allocated_amount, 560);
        assert_eq!(strategy.participants[2].allocated_amount, 240);
        assert_eq!(strategy.participants[3].allocated_amount, 100);
        assert_eq!(strategy.participants[0].last_delta_amount, 0);
        assert_eq!(strategy.participants[1].last_delta_amount, 240);
        assert_eq!(strategy.participants[2].last_delta_amount, 240);
        assert_eq!(strategy.participants[3].last_delta_amount, 0);

        assert_eq!(strategy.cut(2100).unwrap(), 0);
        assert_eq!(strategy.participants[0].allocated_amount, 0);
        assert_eq!(strategy.participants[1].allocated_amount, 0);
        assert_eq!(strategy.participants[2].allocated_amount, 0);
        assert_eq!(strategy.participants[3].allocated_amount, 80);
        assert_eq!(strategy.participants[0].last_delta_amount, -1280);
        assert_eq!(strategy.participants[1].last_delta_amount, -560);
        assert_eq!(strategy.participants[2].last_delta_amount, -240);
        assert_eq!(strategy.participants[3].last_delta_amount, -20);

        assert_eq!(strategy.put(2800).unwrap(), 0);
        assert_eq!(strategy.participants[0].allocated_amount, 1600);
        assert_eq!(strategy.participants[1].allocated_amount, 800);
        assert_eq!(strategy.participants[2].allocated_amount, 400);
        assert_eq!(strategy.participants[3].allocated_amount, 80);
        assert_eq!(strategy.participants[0].last_delta_amount, 1600);
        assert_eq!(strategy.participants[1].last_delta_amount, 800);
        assert_eq!(strategy.participants[2].last_delta_amount, 400);
        assert_eq!(strategy.participants[3].last_delta_amount, 0);

        assert_eq!(strategy.put(1400).unwrap(), 0);
        assert_eq!(strategy.participants[0].allocated_amount, 2000);
        assert_eq!(strategy.participants[1].allocated_amount, 1467);
        assert_eq!(strategy.participants[2].allocated_amount, 733);
        assert_eq!(strategy.participants[3].allocated_amount, 80);
        assert_eq!(strategy.participants[0].last_delta_amount, 400);
        assert_eq!(strategy.participants[1].last_delta_amount, 667);
        assert_eq!(strategy.participants[2].last_delta_amount, 333);
        assert_eq!(strategy.participants[3].last_delta_amount, 0);
    }

    #[test]
    fn test_capped_scenario() {
        let mut strategy = WeightedAllocationStrategy::<4>::new([
            WeightedAllocationParticipant::new(4, 2000, 100),
            WeightedAllocationParticipant::new(2, 0, 100),
            WeightedAllocationParticipant::new(1, 0, 100),
            WeightedAllocationParticipant::new(0, 100, 100),
        ]);

        assert_eq!(strategy.put(2000).unwrap(), 1800);
        assert_eq!(strategy.participants[0].allocated_amount, 2000);
        assert_eq!(strategy.participants[1].allocated_amount, 100);
        assert_eq!(strategy.participants[2].allocated_amount, 100);
        assert_eq!(strategy.participants[3].allocated_amount, 100);
    }
}
