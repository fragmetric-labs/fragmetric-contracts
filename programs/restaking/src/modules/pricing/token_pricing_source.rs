use anchor_lang::prelude::*;

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
    OrcaDEXLiquidityPool {
        address: Pubkey,
    },
    SanctumSingleValidatorSPLStakePool {
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
            Self::OrcaDEXLiquidityPool { address } => {
                write!(f, "OrcaDEXLiquidityPool({})", address)
            }
            Self::SanctumSingleValidatorSPLStakePool { address } => {
                write!(f, "SanctumSingleValidatorSPLStakePool({})", address)
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
            #[cfg(all(test, not(feature = "idl-build")))]
            TokenPricingSource::Mock { .. } => {
                pod.discriminant = 255;
                pod.address = Pubkey::default();
            }
        }
    }
}

#[zero_copy]
#[derive(Debug, Default)]
#[repr(C)]
pub struct TokenPricingSourcePod {
    pub discriminant: u8,
    _padding: [u8; 7],
    pub address: Pubkey,
}

impl TokenPricingSourcePod {
    pub fn clear(&mut self) {
        self.discriminant = 0;
        self.address = Pubkey::default();
    }

    pub fn try_deserialize(&self) -> Result<Option<TokenPricingSource>> {
        Ok({
            if self.discriminant == 0 {
                None
            } else {
                Some(match self.discriminant {
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
                    }
                    7 => TokenPricingSource::SanctumSingleValidatorSPLStakePool {
                        address: self.address,
                    },
                    #[cfg(all(test, not(feature = "idl-build")))]
                    255 => TokenPricingSource::Mock {
                        numerator: vec![],
                        denominator: 1,
                    },
                    _ => Err(Error::from(ProgramError::InvalidAccountData))?,
                })
            }
        })
    }
}
