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
    NormalizedTokenPool {
        mint_address: Pubkey,
        pool_address: Pubkey,
    },
    FundReceiptToken {
        mint_address: Pubkey,
        fund_address: Pubkey,
    },
    #[cfg(test)]
    Mock {
        #[max_len(0)]
        numerator: Vec<MockAsset>,
        denominator: u64,
    },
}
