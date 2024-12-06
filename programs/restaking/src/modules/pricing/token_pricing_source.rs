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

#[derive(Debug)]
#[zero_copy]
pub struct TokenPricingSourcePod {
    discriminant: u8,
    address: Pubkey,
}

impl From<TokenPricingSource> for TokenPricingSourcePod {
    fn from(src: TokenPricingSource) -> Self {
        match src {
            TokenPricingSource::SPLStakePool { address } => Self {
                discriminant: 1,
                address,
            },
            TokenPricingSource::MarinadeStakePool { address } => Self {
                discriminant: 2,
                address,
            },
            TokenPricingSource::JitoRestakingVault { address } => Self {
                discriminant: 3,
                address,
            },
            TokenPricingSource::FragmetricNormalizedTokenPool { address } => Self {
                discriminant: 4,
                address,
            },
            TokenPricingSource::FragmetricRestakingFund { address } => Self {
                discriminant: 5,
                address,
            },
            #[cfg(test)]
            TokenPricingSource::Mock { .. } => Self {
                discriminant: 0,
                address: Pubkey::default(),
            },
        }
    }
}

impl From<TokenPricingSourcePod> for TokenPricingSource {
    fn from(pod: TokenPricingSourcePod) -> Self {
        match pod.discriminant {
            1 => Self::SPLStakePool {
                address: pod.address,
            },
            2 => Self::MarinadeStakePool {
                address: pod.address,
            },
            3 => Self::JitoRestakingVault {
                address: pod.address,
            },
            4 => Self::FragmetricNormalizedTokenPool {
                address: pod.address,
            },
            5 => Self::FragmetricRestakingFund {
                address: pod.address,
            },
            #[cfg(test)]
            0 => Self::Mock {
                denominator: 0,
                numerator: vec![],
            },
            _ => panic!("invalid discriminant for TokenPricingSource"),
        }
    }
}
