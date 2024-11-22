use anchor_lang::prelude::*;

#[cfg(test)]
use crate::modules::pricing::MockAsset;

#[derive(Clone, Debug, InitSpace, AnchorSerialize, AnchorDeserialize, PartialEq)]
#[non_exhaustive]
pub enum TokenPricingSource {
    SPLStakePool {
        address: Pubkey,
    },
    MarinadeStakePool {
        address: Pubkey,
    },
    FragmetricNormalizedTokenPool {
        address: Pubkey,
    },
    FragmetricRestakingFund {
        address: Pubkey,
    },
    #[cfg(test)]
    Mock {
        #[max_len(0)]
        numerator: Vec<MockAsset>,
        denominator: u64,
    },
}
