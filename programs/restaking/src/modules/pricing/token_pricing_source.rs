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
    JitoRestakingVault {
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

impl std::fmt::Display for TokenPricingSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SPLStakePool { address } => write!(f, "SPLStakePool({})", address),
            Self::MarinadeStakePool { address } => write!(f, "MarinadeStakePool({})", address),
            Self::JitoRestakingVault { address } => write!(f, "JitoRestakingVault({})", address),
            Self::FragmetricNormalizedTokenPool { address } => {
                write!(f, "FragmetricNormalizedTokenPool({})", address)
            }
            Self::FragmetricRestakingFund { address } => {
                write!(f, "FragmetricRestakingFund({})", address)
            }
            #[cfg(test)]
            Self::Mock { .. } => write!(f, "Mock(...)"),
        }
    }
}
