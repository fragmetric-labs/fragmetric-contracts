use anchor_lang::prelude::*;

use crate::modules::pricing::TokenPricingSource;

use super::FundAccount;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct NormalizedToken {
    reserved: [u8; 1],
}
