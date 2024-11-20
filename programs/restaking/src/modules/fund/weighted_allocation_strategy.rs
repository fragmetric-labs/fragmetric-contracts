use std::ops::Deref;
use anchor_lang::prelude::*;
use crate::utils;
use crate::errors;
use crate::modules::fund::weighted_allocation_strategy;

#[derive(Clone, Debug)]
pub struct Participant {
    weight: u64,
    allocated_amount: u64,
    capacity_amount: u64,
}

impl Participant {
    fn new(weight: u64, allocated_amount: u64, capacity_amount: u64) -> Self {
        Self { weight, allocated_amount, capacity_amount }
    }

    fn is_full(&self) -> bool {
        self.allocated_amount >= self.capacity_amount
    }
}

pub struct WeightedAllocationStrategy;

impl WeightedAllocationStrategy {
    
    /// returns remaining_resource after the allocation made
    fn put(participants: &mut Vec<Participant>, total_resource: u64) -> u64 {
        let mut remaining_resource = total_resource;

        while remaining_resource > 0 {
            let mut target_participants: Vec<_> = participants.iter_mut()
                .filter(|p| p.weight > 0 && !p.is_full())
                .collect();

            if target_participants.is_empty() {
                break;
            }

            // find the basis participant
            let basis_participant = target_participants.iter()
                .max_by_key(|p| p.allocated_amount)
                .unwrap();

            // calculate shortages
            let shortages = target_participants.iter()
                .map(|p| {
                    let target_amount = utils::get_proportional_amount(basis_participant.allocated_amount, p.weight, basis_participant.weight).unwrap();
                    target_amount.saturating_sub(p.allocated_amount)
                })
                .collect::<Vec<_>>();

            // allocate remaining resources proportionally to shortages first
            let total_shortages = shortages.iter().sum::<u64>();
            if total_shortages > 0 {
                let allocatable_resource = remaining_resource.min(total_shortages);
                let mut allocated_amount = 0;
                for (i, shortage) in shortages.iter().copied().enumerate() {
                    if shortage == 0 {
                        continue;
                    }
                    let p = &mut target_participants[i];
                    let allocating_resource = utils::get_proportional_amount(allocatable_resource, shortage, total_shortages)
                        .unwrap()
                        .min(p.capacity_amount - p.allocated_amount);
                    p.allocated_amount += allocating_resource;
                    allocated_amount += allocating_resource;
                }
                remaining_resource -= allocated_amount;

                // restart allocation
                continue;
            }

            // allocate remaining resources proportionally to weights
            let total_weights = target_participants.iter().map(|p| p.weight).sum();
            let mut allocated_amount = 0;
            for p in target_participants.iter_mut() {
                let allocating_resource = utils::get_proportional_amount(remaining_resource, p.weight, total_weights)
                    .unwrap()
                    .min(p.capacity_amount - p.allocated_amount);
                p.allocated_amount += allocating_resource;
                allocated_amount += allocating_resource;
            }

            if remaining_resource == 1 {
                // cannot allocate more due to precision
                let max_weighted_target_participant = target_participants.iter_mut().filter(|p| !p.is_full()).max_by_key(|p| p.weight);
                if let Some(max_weighted_target_participant) = max_weighted_target_participant {
                    max_weighted_target_participant.allocated_amount += 1;
                    remaining_resource = 0;
                }
                break;

            } else if allocated_amount == 0 {
                // cannot allocate more due to maxed capacity
                break
            }

            remaining_resource -= allocated_amount;
        }

        remaining_resource
    }

    /// returns required_resource after the de-allocation made
    fn cut(participants: &mut Vec<Participant>, total_resource: u64) -> u64 {
        let mut required_resource = total_resource;

        // cut from non-zero weighted participants first
        let weighted_participants = &mut participants.iter_mut().filter(|p| p.weight > 0).collect::<Vec<_>>();
        weighted_participants.sort_by_key(|p| p.weight);
        for p in weighted_participants.iter_mut() {
            if required_resource == 0 {
                break;
            }
            if p.weight > 0 {
                let deallocating_resource = required_resource.min(p.allocated_amount);
                p.allocated_amount -= deallocating_resource;
                required_resource -= deallocating_resource;
            }
        }

        // cut from zero weighted participants if needed
        if required_resource > 0 {
            let non_weighted_participants = &mut participants.iter_mut().filter(|p| p.weight == 0).collect::<Vec<_>>();
            let total_allocated_amount = non_weighted_participants.iter().map(|p| p.allocated_amount).sum();

            let mut deallocated_resource = 0;
            for p in non_weighted_participants.iter_mut() {
                let deallocating_resource = utils::get_proportional_amount(required_resource, p.allocated_amount, total_allocated_amount)
                    .unwrap()
                    .min(p.allocated_amount);
                p.allocated_amount -= deallocating_resource;
                deallocated_resource += deallocating_resource;
            }
            required_resource -= deallocated_resource;
        }

        required_resource
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_general_scenario() {
        let participants = &mut vec![
            Participant::new(4, 0, 2000),
            Participant::new(2, 0, u64::MAX),
            Participant::new(1, 0, u64::MAX),
            Participant::new(0, 0, u64::MAX),
        ];

        // step-by-step allocations and cuts
        participants[3].allocated_amount = 100;
        assert_eq!(participants[3].allocated_amount, 100);

        assert_eq!(WeightedAllocationStrategy::put(participants, 700), 0);
        assert_eq!(participants[0].allocated_amount, 400);
        assert_eq!(participants[1].allocated_amount, 200);
        assert_eq!(participants[2].allocated_amount, 100);

        assert_eq!(WeightedAllocationStrategy::put(participants, 700), 0);
        assert_eq!(participants[0].allocated_amount, 800);
        assert_eq!(participants[1].allocated_amount, 400);
        assert_eq!(participants[2].allocated_amount, 200);

        // direct allocation: put 400 to jitoSOL
        participants[0].allocated_amount += 400;
        assert_eq!(participants[0].allocated_amount, 1200);
        assert_eq!(participants[1].allocated_amount, 400);
        assert_eq!(participants[2].allocated_amount, 200);

        // direct allocation: put 50 to bSOL
        participants[2].allocated_amount += 50;
        assert_eq!(participants[0].allocated_amount, 1200);
        assert_eq!(participants[1].allocated_amount, 400);
        assert_eq!(participants[2].allocated_amount, 250);

        assert_eq!(WeightedAllocationStrategy::put(participants, 320), 0);
        assert_eq!(participants[0].allocated_amount, 1240);
        assert_eq!(participants[1].allocated_amount, 620);
        assert_eq!(participants[2].allocated_amount, 310);

        assert_eq!(WeightedAllocationStrategy::cut(participants, 300), 0);
        assert_eq!(participants[0].allocated_amount, 1240);
        assert_eq!(participants[1].allocated_amount, 620);
        assert_eq!(participants[2].allocated_amount, 10);

        assert_eq!(WeightedAllocationStrategy::put(participants, 370), 0);
        assert_eq!(participants[0].allocated_amount, 1280);
        assert_eq!(participants[1].allocated_amount, 640);
        assert_eq!(participants[2].allocated_amount, 320);

        assert_eq!(WeightedAllocationStrategy::cut(participants, 640), 0);
        assert_eq!(participants[0].allocated_amount, 1280);
        assert_eq!(participants[1].allocated_amount, 320);
        assert_eq!(participants[2].allocated_amount, 0);

        assert_eq!(WeightedAllocationStrategy::put(participants, 480), 0);
        assert_eq!(participants[0].allocated_amount, 1280);
        assert_eq!(participants[1].allocated_amount, 560);
        assert_eq!(participants[2].allocated_amount, 240);

        assert_eq!(WeightedAllocationStrategy::cut(participants, 2100), 0);
        assert_eq!(participants[0].allocated_amount, 0);
        assert_eq!(participants[1].allocated_amount, 0);
        assert_eq!(participants[2].allocated_amount, 0);
        assert_eq!(participants[3].allocated_amount, 80);

        assert_eq!(WeightedAllocationStrategy::put(participants, 2800), 0);
        assert_eq!(participants[0].allocated_amount, 1600);
        assert_eq!(participants[1].allocated_amount, 800);
        assert_eq!(participants[2].allocated_amount, 400);
        assert_eq!(participants[3].allocated_amount, 80);

        assert_eq!(WeightedAllocationStrategy::put(participants, 1400), 0);
        assert_eq!(participants[0].allocated_amount, 2000);
        assert_eq!(participants[1].allocated_amount, 1467);
        assert_eq!(participants[2].allocated_amount, 733);
        assert_eq!(participants[3].allocated_amount, 80);
    }

    #[test]
    fn test_capped_scenario() {
        let participants = &mut vec![
            Participant::new(4, 2000, 100),
            Participant::new(2, 0, 100),
            Participant::new(1, 0, 100),
            Participant::new(0, 100, 100),
        ];

        assert_eq!(WeightedAllocationStrategy::put(participants, 2000), 1800);
        assert_eq!(participants[0].allocated_amount, 2000);
        assert_eq!(participants[1].allocated_amount, 100);
        assert_eq!(participants[2].allocated_amount, 100);
        assert_eq!(participants[3].allocated_amount, 100);
    }
}