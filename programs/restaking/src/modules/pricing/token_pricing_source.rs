use anchor_lang::prelude::*;

#[cfg(all(test, not(feature = "idl-build")))]
use super::MockAsset;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, PartialEq)]
#[cfg_attr(test, derive(Debug))]
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
    OrcaDEXLiquidityPool {
        address: Pubkey,
    },
    SanctumSingleValidatorSPLStakePool {
        address: Pubkey,
    },
    PeggedToken {
        address: Pubkey,
    },
    SolvBTCVault {
        address: Pubkey,
    },
    SanctumMultiValidatorSPLStakePool {
        address: Pubkey,
    },
    VirtualVault {
        address: Pubkey,
    },
    #[cfg(all(test, not(feature = "idl-build")))]
    Mock {
        #[max_len(0)]
        numerator: Vec<MockAsset>,
        denominator: u64,
    },
}

impl core::fmt::Display for TokenPricingSource {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
            Self::OrcaDEXLiquidityPool { address } => {
                write!(f, "OrcaDEXLiquidityPool({})", address)
            }
            Self::SanctumSingleValidatorSPLStakePool { address } => {
                write!(f, "SanctumSingleValidatorSPLStakePool({})", address)
            }
            Self::PeggedToken { address } => {
                write!(f, "PeggedToken({})", address)
            }
            Self::SolvBTCVault { address } => {
                write!(f, "SolvBTCVault({})", address)
            }
            Self::SanctumMultiValidatorSPLStakePool { address } => {
                write!(f, "SanctumMultiValidatorSPLStakePool({})", address)
            }
            Self::VirtualVault { address } => {
                write!(f, "VirtualVault({})", address)
            }
            #[cfg(all(test, not(feature = "idl-build")))]
            Self::Mock { .. } => write!(f, "Mock(...)"),
        }
    }
}

impl TokenPricingSource {
    pub fn serialize_as_pod(&self, pod: &mut TokenPricingSourcePod) {
        match self {
            TokenPricingSource::SPLStakePool { address } => {
                pod.discriminant = 1;
                pod.address = *address;
            }
            TokenPricingSource::MarinadeStakePool { address } => {
                pod.discriminant = 2;
                pod.address = *address;
            }
            TokenPricingSource::JitoRestakingVault { address } => {
                pod.discriminant = 3;
                pod.address = *address;
            }
            TokenPricingSource::FragmetricNormalizedTokenPool { address } => {
                pod.discriminant = 4;
                pod.address = *address;
            }
            TokenPricingSource::FragmetricRestakingFund { address } => {
                pod.discriminant = 5;
                pod.address = *address;
            }
            TokenPricingSource::OrcaDEXLiquidityPool { address } => {
                pod.discriminant = 6;
                pod.address = *address;
            }
            TokenPricingSource::SanctumSingleValidatorSPLStakePool { address } => {
                pod.discriminant = 7;
                pod.address = *address;
            }
            TokenPricingSource::PeggedToken { address } => {
                pod.discriminant = 8;
                pod.address = *address;
            }
            TokenPricingSource::SolvBTCVault { address } => {
                pod.discriminant = 9;
                pod.address = *address;
            }
            TokenPricingSource::SanctumMultiValidatorSPLStakePool { address } => {
                pod.discriminant = 10;
                pod.address = *address;
            }
            TokenPricingSource::VirtualVault { address } => {
                pod.discriminant = 11;
                pod.address = *address;
            }
            #[cfg(all(test, not(feature = "idl-build")))]
            TokenPricingSource::Mock { .. } => {
                pod.discriminant = 255;
                pod.address = Pubkey::default();
            }
        }
    }
}

/// Pod type of `Option<TokenPricingSource>`
#[zero_copy]
#[repr(C)]
pub struct TokenPricingSourcePod {
    discriminant: u8,
    _padding: [u8; 7],
    address: Pubkey,
}

impl TokenPricingSourcePod {
    pub fn address(&self) -> Option<Pubkey> {
        (self.discriminant != 0).then_some(self.address)
    }

    pub fn set_none(&mut self) {
        self.discriminant = 0;
        self.address = Pubkey::default();
    }

    pub fn is_none(&self) -> bool {
        self.discriminant == 0
    }

    pub fn try_deserialize(&self) -> Result<Option<TokenPricingSource>> {
        Ok(Some(match self.discriminant {
            0 => return Ok(None),
            1 => TokenPricingSource::SPLStakePool {
                address: self.address,
            },
            2 => TokenPricingSource::MarinadeStakePool {
                address: self.address,
            },
            3 => TokenPricingSource::JitoRestakingVault {
                address: self.address,
            },
            4 => TokenPricingSource::FragmetricNormalizedTokenPool {
                address: self.address,
            },
            5 => TokenPricingSource::FragmetricRestakingFund {
                address: self.address,
            },
            6 => TokenPricingSource::OrcaDEXLiquidityPool {
                address: self.address,
            },
            7 => TokenPricingSource::SanctumSingleValidatorSPLStakePool {
                address: self.address,
            },
            8 => TokenPricingSource::PeggedToken {
                address: self.address,
            },
            9 => TokenPricingSource::SolvBTCVault {
                address: self.address,
            },
            10 => TokenPricingSource::SanctumMultiValidatorSPLStakePool {
                address: self.address,
            },
            11 => TokenPricingSource::VirtualVault {
                address: self.address,
            },
            #[cfg(all(test, not(feature = "idl-build")))]
            255 => TokenPricingSource::Mock {
                numerator: vec![],
                denominator: 1,
            },
            _ => Err(Error::from(ProgramError::InvalidAccountData))?,
        }))
    }
}
