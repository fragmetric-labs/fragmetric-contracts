use anchor_lang::prelude::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, PartialEq)]
#[non_exhaustive]
pub enum TokenSwapSource {
    OrcaDEXLiquidityPool { address: Pubkey },
}

impl TokenSwapSource {
    pub fn serialize_as_pod(&self, pod: &mut TokenSwapSourcePod) {
        match self {
            Self::OrcaDEXLiquidityPool { address } => {
                pod.discriminant = 1;
                pod.address = *address;
            }
        }
    }
}

/// Pod type of `TokenSwapSource`
#[zero_copy]
#[repr(C)]
pub struct TokenSwapSourcePod {
    discriminant: u8,
    _padding: [u8; 7],
    address: Pubkey,
}

impl TokenSwapSourcePod {
    pub fn address(&self) -> Pubkey {
        self.address
    }

    pub fn try_deserialize(&self) -> Result<TokenSwapSource> {
        Ok(match self.discriminant {
            1 => TokenSwapSource::OrcaDEXLiquidityPool {
                address: self.address,
            },
            _ => Err(Error::from(ProgramError::InvalidAccountData))?,
        })
    }
}
