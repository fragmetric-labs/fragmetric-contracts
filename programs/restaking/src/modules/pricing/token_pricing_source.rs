use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

#[cfg(all(test, not(feature = "idl-build")))]
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
    #[cfg(all(test, not(feature = "idl-build")))]
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
            #[cfg(all(test, not(feature = "idl-build")))]
            Self::Mock { .. } => write!(f, "Mock(...)"),
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod, Debug, Default)]
#[repr(C)]
pub struct TokenPricingSourcePod {
    discriminant: u8,
    _padding: [u8; 7],
    address: Pubkey,
}

impl TokenPricingSourcePod {
    pub fn try_deserialize(&self) -> Result<TokenPricingSource> {
        self.try_into()
    }
}

impl From<TokenPricingSource> for TokenPricingSourcePod {
    fn from(src: TokenPricingSource) -> Self {
        match src {
            TokenPricingSource::SPLStakePool { address } => Self {
                discriminant: 1,
                _padding: [0; 7],
                address,
            },
            TokenPricingSource::MarinadeStakePool { address } => Self {
                discriminant: 2,
                _padding: [0; 7],
                address,
            },
            TokenPricingSource::JitoRestakingVault { address } => Self {
                discriminant: 3,
                _padding: [0; 7],
                address,
            },
            TokenPricingSource::FragmetricNormalizedTokenPool { address } => Self {
                discriminant: 4,
                _padding: [0; 7],
                address,
            },
            TokenPricingSource::FragmetricRestakingFund { address } => Self {
                discriminant: 5,
                _padding: [0; 7],
                address,
            },
            #[cfg(all(test, not(feature = "idl-build")))]
            TokenPricingSource::Mock { .. } => Self {
                discriminant: 255,
                _padding: [0; 7],
                address: Pubkey::default(),
            },
        }
    }
}

impl TryFrom<&TokenPricingSourcePod> for TokenPricingSource {
    type Error = anchor_lang::error::Error;

    fn try_from(pod: &TokenPricingSourcePod) -> Result<TokenPricingSource> {
        Ok(match pod.discriminant {
            1 => TokenPricingSource::SPLStakePool {
                address: pod.address,
            },
            2 => TokenPricingSource::MarinadeStakePool {
                address: pod.address,
            },
            3 => TokenPricingSource::JitoRestakingVault {
                address: pod.address,
            },
            4 => TokenPricingSource::FragmetricNormalizedTokenPool {
                address: pod.address,
            },
            5 => TokenPricingSource::FragmetricRestakingFund {
                address: pod.address,
            },
            #[cfg(all(test, not(feature = "idl-build")))]
            255 => TokenPricingSource::Mock {
                denominator: 255,
                numerator: vec![],
            },
            _ => Err(Error::from(ProgramError::InvalidAccountData))?,
        })
    }
}
